//! Checked blocking source requests with explicit configuration and deadlines.
#[cfg(test)]
extern crate self as biomcp_fetch;

mod cancellation;
pub mod grpc;
mod observe;
pub use cancellation::Cancellation;
mod observe_context;
pub use observe::{MAX_OBSERVATIONS, Observation, Observations, Shape};
pub use observe_context::{Identities, Provenance};
mod batch;
mod bundle;
mod cache;
mod cache_access;
mod trial_cursor;
pub use trial_cursor::TrialCursorKey;
mod cleanup;
mod sqlite_diagnostic;
mod store_size;
pub use cache_access::CacheAccess;
#[doc(hidden)]
pub use observe::record_sqlite_error;
#[doc(hidden)]
pub use sqlite_diagnostic::SQLITE_DIAGNOSTICS;
mod cache_request;
mod containment;
mod deadline;
mod declaration;
mod destination;
pub use destination::check_annotation_url;
mod hosts;
pub use hosts::Destinations;
mod error;
mod network;
mod prepared;
mod privacy;
mod tls;
pub use network::{Certificates, Network};
pub use tls::TlsCause;
mod quota;
mod read;
mod record;
mod request;
pub use read::Read;
mod call;
pub use call::{Absent, Input, JsonBody, Received};
pub use xml::Bytes;
mod retry;
mod secret;
mod wire_request;
pub mod xml;

/// Original JSON row bytes carried through a typed transport document boundary.
pub use serde_json::value::RawValue;

pub use batch::MAX_BATCH_WORKERS;
pub use bundle::{
    Bundle, BundleInfo, BundlePart, BundleRejection, BundleSet, BundleSync, DocumentMedia, Find,
    KeyKind, Keys, MAX_BUNDLE_BYTES, Row, RowFormat, SetRow, bundle_sha256, decode_bundle_document,
    encode_bundle_row,
};
pub use cache::sql_deadline as configure_sqlite_deadline;
pub use cache::{CACHE_NAMESPACE_VERSION, STALE_GRACE_SECONDS, SchemaOwner, VERSION, check_owners};
pub use deadline::{CommandDeadline, Deadline, LocalDeadline};
pub use declaration::{
    Arg, Credential, Endpoint, Freshness, KeyPlace, KeyUse, PathSegment, SecretName, SourceSpec,
    Terms,
};
pub use error::{
    BadReply, CacheError, FetchError, InputError, RecordError, RequestError, SourceError, bad_reply,
};
pub use quota::{
    InvalidRateShare, PacingOverride, PacingRate, Quota, Rate, RateShare, SourceKey, SourcePacing,
};
pub use record::ReplayCache;
pub use secret::{KeyFingerprint, Secret, Secrets};
use std::{collections::BTreeMap, fmt, time::Duration};

/// A decoded response must verify that it contains a usable answer.
pub trait Reply: serde::de::DeserializeOwned {
    /// Refuse provider error envelopes and incomplete answers.
    fn check(&self) -> Result<(), BadReply>;
    /// Publisher dataset date, distinct from record events and local retrieval.
    fn data_date(&self) -> Option<String> {
        None
    }
    /// Source-given identities used to discover dependent requests, only for explicit reply types.
    fn discovery(&self) -> Option<Identities> {
        None
    }
    /// Complete checked provider sequences when discovery spans identifier namespaces.
    fn discoveries(&self) -> Option<Vec<Identities>> {
        None
    }
    /// Explicit namespace and ID parameter used by this dependent request.
    fn input_parameter() -> Option<(&'static str, &'static str)> {
        None
    }
    /// Refuse a successful-looking body on an error status. Explicit validation-reply types may accept only their checked rejection form.
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if (200..300).contains(&status) {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}

/// Explicit transport configuration. Construction reads no environment settings.
#[derive(Debug)]
pub struct FetchConfig {
    /// This process's share of provider rates in hosted mode.
    pub rate_share: RateShare,
    /// Checked source-specific rates and simultaneous request counts.
    pub source_pacing: SourcePacing,
    /// Bounded streaming RPC resources, independent of blocking HTTP requests.
    pub grpc_limits: grpc::Limits,
    /// Explicit bounded wire summaries, disabled by default.
    pub observations: Option<std::sync::Arc<Observations>>,
    /// Other schema owners checked before any cache mutation.
    pub cache_owners: Vec<SchemaOwner>,
    /// Source identities switched off before keys, caches, replay or requests.
    pub disabled_sources: std::collections::BTreeSet<biomcp_types::Source>,
    /// Component terms categories switched off by the same source policy.
    pub disabled_uses: std::collections::BTreeSet<biomcp_types::Use>,
    /// Configured credentials.
    pub secrets: Secrets,
    /// Explicit SQLite cache path; absence disables storage.
    pub cache: Option<std::path::PathBuf>,
    /// Explicit older files checked only when an allowed bundle read misses.
    pub cache_predecessors: Vec<std::path::PathBuf>,
    /// Live request policy.
    pub mode: Mode,
    /// Owned case cache for replay writes or later offline reads.
    pub replay_cache: Option<ReplayCache>,
    /// Explicit HTTP proxy URL, redacted when formatted.
    pub proxy: Option<Secret>,
    /// Checked environment routes and additive trust certificates.
    pub network: Network,
    /// Maximum encoded request and decoded response bytes, each bounded independently.
    pub max_body: u64,
    /// Independent explicit bundle synchronization budget.
    pub sync_budget: Duration,
    /// Maximum automatically downloaded bundle bytes.
    pub auto_sync_max_bytes: u64,
    /// Days after a publisher dataset date before replies flag old data.
    pub source_data_max_age_days: u32,
    /// Per-source budget overrides.
    pub source_deadlines: BTreeMap<biomcp_types::Source, Duration>,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            rate_share: RateShare::default(),
            source_pacing: SourcePacing::default(),
            grpc_limits: grpc::Limits::default(),
            observations: None,
            cache_owners: Vec::new(),
            disabled_sources: Default::default(),
            disabled_uses: Default::default(),
            secrets: Secrets::default(),
            cache: None,
            cache_predecessors: Vec::new(),
            mode: Mode::default(),
            replay_cache: None,
            proxy: None,
            network: Network::default(),
            max_body: 8 * 1024 * 1024,
            source_deadlines: BTreeMap::new(),
            sync_budget: Duration::from_secs(120),
            auto_sync_max_bytes: 32 * 1024 * 1024,
            source_data_max_age_days: 365,
        }
    }
}

/// One blocking client shared across source legs.
#[expect(
    clippy::disallowed_types,
    reason = "Reserve each bundle replay import once per Fetch"
)]
#[derive(Clone)]
pub struct Fetch {
    secrets: std::sync::Arc<Secrets>,
    trial_cursor_key: Option<std::sync::Arc<TrialCursorKey>>,
    hosted: bool,
    agents: std::sync::Arc<network::Agents>,
    tls: std::sync::Arc<rustls::ClientConfig>,
    grpc_worker: std::sync::Arc<grpc::Worker>,
    cancelled: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    call_end: Option<std::time::Instant>,
    mode: Mode,
    limiters: std::sync::Arc<quota::Limiters>,
    cache: Option<std::sync::Arc<cache::Cache>>,
    recordings: std::sync::Arc<record::Recordings>,
    clock: Option<std::sync::Arc<deadline::VirtualClock>>,
    config: std::sync::Arc<FetchConfig>,
    background_refresh: Option<std::sync::Arc<BundleRefresh>>,
    bundle_imports: std::sync::Arc<std::sync::Mutex<std::collections::BTreeSet<&'static str>>>,
    disabled_sources: std::collections::BTreeSet<biomcp_types::Source>,
    disabled_uses: std::collections::BTreeSet<biomcp_types::Use>,
    commercial_studies_only: bool,
}

#[expect(
    clippy::disallowed_types,
    reason = "The server owns and joins at most one refresh worker per declared bundle"
)]
struct BundleRefresh {
    cancellation: Cancellation,
    running:
        std::sync::Mutex<std::collections::BTreeMap<&'static str, std::thread::JoinHandle<()>>>,
}
impl fmt::Debug for Fetch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fetch")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Fetch {
    /// Retain automatic stale refreshes until the last server transport is dropped.
    /// One-shot commands keep their synchronous read behavior.
    #[expect(
        clippy::disallowed_types,
        reason = "The server owns and joins at most one refresh worker per declared bundle"
    )]
    pub fn with_background_refresh(mut self) -> Self {
        if self.background_refresh.is_none() {
            self.background_refresh = Some(std::sync::Arc::new(BundleRefresh {
                cancellation: Cancellation::new(None),
                running: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            }));
        }
        self
    }
    /// Bounded nonfatal settings notices for local health diagnostics.
    pub fn configuration_health(&self) -> Option<&biomcp_types::ConfigurationHealth> {
        self.config.network.configuration()
    }
    /// Open a client using platform TLS roots and only the supplied proxy.
    pub fn open(mut config: FetchConfig) -> Result<Self, FetchError> {
        if config.max_body == u64::MAX {
            return Err(FetchError::BodyLimit);
        }
        let tls = std::sync::Arc::new(config.network.tls_configuration()?);
        let grpc_worker = std::sync::Arc::new(grpc::Worker::new(tls.clone(), config.grpc_limits)?);
        let agents = network::Agents::new(&config.network, config.proxy.as_ref(), tls.clone())?;
        if matches!(config.mode, Mode::Replay { .. }) && config.replay_cache.is_none() {
            config.replay_cache = Some(
                ReplayCache::temporary_with_owners(&config.cache_owners)
                    .map_err(FetchError::Cache)?,
            );
        }
        let recordings = record::Recordings::open(&config.mode).map_err(FetchError::Recording)?;
        let cache = if matches!(config.mode, Mode::Replay { .. } | Mode::Offline)
            && config.replay_cache.is_some()
        {
            config.replay_cache.as_ref().map(|cache| cache.cache())
        } else if matches!(config.mode, Mode::Replay { .. }) {
            None
        } else {
            config
                .cache
                .as_deref()
                .map(|path| cache::Cache::open_with_owners(path, &config.cache_owners))
                .transpose()
                .map_err(FetchError::Cache)?
                .map(std::sync::Arc::new)
        };
        let clock =
            matches!(config.mode, Mode::Replay { .. }).then(deadline::VirtualClock::default);
        Ok(Self {
            secrets: std::sync::Arc::new(std::mem::take(&mut config.secrets)),
            trial_cursor_key: None,
            hosted: false,
            agents: std::sync::Arc::new(agents),
            grpc_worker,
            tls,
            cancelled: None,
            call_end: None,
            mode: std::mem::take(&mut config.mode),
            cache,
            recordings: std::sync::Arc::new(recordings),
            clock: clock.map(std::sync::Arc::new),
            disabled_sources: config.disabled_sources.clone(),
            disabled_uses: config.disabled_uses.clone(),
            commercial_studies_only: false,
            config: std::sync::Arc::new(config),
            limiters: std::sync::Arc::default(),
            background_refresh: None,
            bundle_imports: std::sync::Arc::default(),
        })
    }

    /// Whether this command explicitly forbids upstream requests.
    pub fn is_offline(&self) -> bool {
        matches!(self.mode, Mode::Offline)
    }
    /// Whether the explicit source policy prevents every read and request.
    pub fn disabled(&self, source: biomcp_types::Source) -> bool {
        self.disabled_sources.contains(&source)
    }
    /// Preserve every explicit source restriction in generated follow-up commands.
    pub fn disabled_sources(&self) -> impl Iterator<Item = biomcp_types::Source> + '_ {
        self.disabled_sources.iter().copied()
    }

    /// Restrict a command while retaining shared quotas, clocks, recordings and connections.
    /// Existing source restrictions cannot be removed.
    pub fn restrict_sources(
        &self,
        disabled: std::collections::BTreeSet<biomcp_types::Source>,
    ) -> Self {
        let mut restricted = self.clone();
        restricted.disabled_sources.extend(disabled);
        restricted
    }

    /// Whether a component category was explicitly switched off.
    pub fn disabled_use(&self, usage: biomcp_types::Use) -> bool {
        self.disabled_uses.contains(&usage)
    }
    /// Preserve component group restrictions in generated follow-up commands.
    pub fn disabled_uses(&self) -> impl Iterator<Item = biomcp_types::Use> + '_ {
        self.disabled_uses.iter().copied()
    }
    /// Add component restrictions without changing source policy or shared resources.
    pub fn restrict_uses(&self, disabled: std::collections::BTreeSet<biomcp_types::Use>) -> Self {
        let mut restricted = self.clone();
        restricted.disabled_uses.extend(disabled);
        restricted
    }

    /// Apply the restricted hosted mode's study-level commercial-use policy.
    pub fn restrict_studies(&self) -> Self {
        let mut restricted = self.clone();
        restricted.commercial_studies_only = true;
        restricted
    }
    /// Whether reviewed study restrictions must be enforced on this request.
    pub fn commercial_studies_only(&self) -> bool {
        self.commercial_studies_only
    }

    /// Replace this command's credentials while retaining every shared transport resource.
    /// Other concurrent calls keep their own immutable credential snapshot.
    pub fn with_secrets(&self, secrets: Secrets) -> Self {
        let mut fetch = self.clone();
        fetch.secrets = std::sync::Arc::new(secrets);
        fetch
    }

    /// Report whether a configured credential is nonempty without exposing it.
    pub fn has_key(&self, name: SecretName) -> bool {
        self.secrets.get(name).is_some()
    }

    /// Create a real deadline or an initial replay budget for independent legs.
    pub fn deadline(&self, budget: Duration) -> Deadline<'_> {
        Deadline::start(self.bounded_budget(budget), self.clock.as_deref())
            .with_cancellation(self.cancelled.as_ref())
    }

    /// Own a sequential command's replay origin without changing independent leg consumers.
    pub fn command_deadline(&self, budget: Duration) -> CommandDeadline<'_> {
        CommandDeadline::start(self.bounded_budget(budget), self.clock.as_deref())
            .with_cancellation(self.cancelled.as_ref())
    }
    fn bounded_budget(&self, budget: Duration) -> Duration {
        self.call_end.map_or(budget, |end| {
            budget.min(end.saturating_duration_since(std::time::Instant::now()))
        })
    }

    /// Start a leg capped by this source's budget.
    /// Replay legs spend independent virtual budgets and share provider quota schedules.
    /// Requests reject deadlines from a different clock domain or replay client.
    #[expect(
        clippy::disallowed_types,
        reason = "Create the shared leg report lock without holding it during request work"
    )]
    pub fn leg<'a>(&'a self, spec: &'static SourceSpec, deadline: Deadline<'a>) -> Leg<'a> {
        let cap = self
            .config
            .source_deadlines
            .get(&spec.source)
            .copied()
            .unwrap_or(spec.deadline);
        let no_key = !self.disabled(spec.source)
            && match spec.key {
                KeyUse::Required(credential) => self.secrets.get(credential.secret).is_none(),
                _ => false,
            };
        Leg {
            fetch: self,
            spec,
            deadline: deadline.child(cap),
            clock: deadline.fork(),
            wrong_clock: !deadline.matches(self.clock.as_deref()),
            no_key,
            report: std::sync::Arc::new(std::sync::Mutex::new(LegReport::default())),
            nested_batch: false,
            refreshed_bundles: Default::default(),
            fresh: matches!(self.mode, Mode::Live { fresh: true }),
        }
    }
}

/// One source's requests under a shared absolute deadline.
#[expect(
    clippy::disallowed_types,
    reason = "Keep typed report state shared across concurrent requests; never hold its lock during request work"
)]
#[derive(Debug)]
pub struct Leg<'a> {
    fetch: &'a Fetch,
    spec: &'static SourceSpec,
    deadline: Deadline<'a>,
    clock: Option<deadline::VirtualClock>,
    wrong_clock: bool,
    no_key: bool,
    report: std::sync::Arc<std::sync::Mutex<LegReport>>,
    nested_batch: bool,
    fresh: bool,
    refreshed_bundles: std::sync::Arc<std::sync::Mutex<std::collections::BTreeSet<&'static str>>>,
}

/// Request policy for live provider access.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Capture each provider response in a new or empty directory.
    Record {
        /// Exclusive capture directory.
        dir: std::path::PathBuf,
    },
    /// Read only captured exchanges with virtual time.
    Replay {
        /// Existing recording directory.
        dir: std::path::PathBuf,
    },
    /// Serve only checked cache entries, including stale positive replies.
    Offline,
    /// Request a provider using the checked cache policy.
    Live {
        /// Skip cached reads and stale fallback.
        fresh: bool,
    },
}
impl Default for Mode {
    fn default() -> Self {
        Self::Live { fresh: false }
    }
}

/// The retained stale state of a completed source leg.
#[derive(Debug, Default)]
pub struct LegReport {
    /// Publisher dataset dates, age assessments and fixed cache-knowledge guidance.
    pub data_notices: Vec<biomcp_types::SourceNotice>,
    /// Body bytes read from HTTP in this leg, excluding cache and replay.
    pub(crate) downloaded_bytes: u64,
    /// Successful replies with their canonical identity and original fetch time.
    pub replies: Vec<ReplyMetadata>,
    /// Whether any request served a checked stale reply.
    pub stale: bool,
    /// Typed failures behind stale replies, and the reason a bundle read stopped on a busy store.
    pub failures: Vec<SourceError>,
    /// Cache failures that did not invalidate the checked provider answer.
    pub cache_problems: Vec<CacheError>,
    /// A license lookup did not settle every paper, so this leg stored no reply.
    pub license_unchecked: bool,
}
mod data_age;
impl Leg<'_> {
    /// Keep this leg's replies out of the cache: text filtered on an unfinished
    /// license lookup is not the source's answer and cannot recover once stored.
    pub fn note_license_unchecked(&self) {
        self.report
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .license_unchecked = true;
    }
    pub(crate) fn license_unchecked(&self) -> bool {
        self.report
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .license_unchecked
    }
    /// Run a metadata dependency within the remaining request budget.
    /// Charge its independent replay clock back to the calling source leg.
    pub fn dependency<T>(
        &self,
        spec: &'static SourceSpec,
        cap: Duration,
        operation: impl FnOnce(&Leg<'_>) -> T,
    ) -> T {
        let mut leg = self.fetch.leg(spec, self.deadline);
        leg.deadline = leg.deadline.child(self.remaining().min(cap));
        let started = leg.remaining();
        let result = operation(&leg);
        self.budget()
            .advance(started.saturating_sub(leg.remaining()));
        result
    }
    /// Inspect the same source policy without opening another transport leg.
    pub fn disabled_source(&self, source: biomcp_types::Source) -> bool {
        self.fetch.disabled(source)
    }
    /// Inspect the same component policy before choosing a source projection.
    pub fn disabled_use(&self, usage: biomcp_types::Use) -> bool {
        self.fetch.disabled_use(usage)
    }
    /// Bypass cached reads and stale fallback for this leg and its batch children.
    /// The process policy cannot be weakened by a call.
    pub fn fresh(mut self, fresh: bool) -> Self {
        self.fresh |= fresh;
        self
    }
    pub(crate) fn budget(&self) -> Deadline<'_> {
        self.deadline.bind(self.clock.as_ref())
    }
    /// Bind the source leg's remaining budget for cooperative local parsing.
    pub fn local_deadline(&self) -> LocalDeadline {
        self.budget().local()
    }
    /// Return this source leg's remaining real or virtual budget.
    pub fn remaining(&self) -> Duration {
        self.budget().remaining()
    }
    /// Finish the leg without clearing failures after later successful requests.
    pub fn finish(self) -> LegReport {
        self.take_report()
    }
    /// Drain one read's report while retaining the source deadline and refreshed bundles.
    /// Call only after all requests contributing to that read have joined.
    pub fn take_report(&self) -> LegReport {
        std::mem::take(
            &mut *self
                .report
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        )
    }
}

/// Provenance of one checked response, independent of other replies in its leg.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplyMetadata {
    /// Credential-free request identity.
    pub identity: [u8; 32],
    /// Original fetch time in Unix seconds, absent in legacy recordings without a timestamp.
    pub fetched: Option<i64>,
    /// Native exact count and selected bounds, when supplied by the provider.
    pub content_range: Option<ContentRange>,
    /// Bounded native count and next link, only on UniProt search replies.
    pub pagination: Option<Pagination>,
}

mod pagination;
pub use pagination::Pagination;
mod counted;
pub use counted::ContentRange;

mod request_body;

#[cfg(test)]
#[path = "../tests/transport/tls_builds.rs"]
mod tls_build_tests;
