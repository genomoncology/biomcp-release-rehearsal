//! Source outcomes retain publisher metadata and one atomic status.
use crate::{Source, SourceStatus, SourceStatusView};
use serde::{Deserialize, Serialize};
biodata::wire_enum! {
/// State of a source actually asked by the command.
SourceState {
    /// Checked data is current.
    Ok => "ok",
    /// Checked stale data answered.
    Stale => "stale",
    /// Another process is synchronizing the bundle.
    Syncing => "syncing",
    /// The source failed.
    Failed => "failed",
    /// User settings prevented the source request.
    Skipped => "skipped",
}
}
/// One source's outcome, without provider text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SourceOutcome {
    /// Asked source.
    pub source: Source,
    /// Reviewed source use category; publisher terms remain authoritative.
    #[serde(rename = "use")]
    pub usage: crate::Use,
    /// Publisher terms name from the source registry.
    pub terms: Box<str>,
    /// Publisher page documenting the terms.
    #[cfg_attr(feature = "schema", schemars(extend("format" = "uri")))]
    pub terms_url: Box<str>,
    #[serde(flatten)]
    status: SourceStatusView,
    /// Exact provider search term.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<crate::Text>,
    /// Provider matching count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    /// Number of retrieved identities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_count: Option<u32>,
    /// Source notices and request recovery guidance.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<SourceNotice>,
}

impl SourceOutcome {
    /// Build a source report with one complete status.
    pub fn new(
        source: Source,
        usage: crate::Use,
        terms: Box<str>,
        terms_url: Box<str>,
        status: SourceStatus,
    ) -> Self {
        Self {
            source,
            usage,
            terms,
            terms_url,
            status: status.into(),
            term: None,
            count: None,
            retrieved_count: None,
            notices: Vec::new(),
        }
    }

    /// Replace state and reason together.
    ///
    /// Individual fields remain read-only, including on a mutable outcome.
    /// ```compile_fail
    /// fn invalid(mut source: biomcp_types::SourceOutcome) {
    ///     source.reason = Some(biomcp_types::Reason::Unsupported);
    /// }
    /// ```
    /// A copied read-only view cannot be installed as a status.
    /// ```compile_fail
    /// fn invalid(mut source: biomcp_types::SourceOutcome) {
    ///     let mut view = (*source).clone();
    ///     view.reason = Some(biomcp_types::Reason::Unsupported);
    ///     source.set_status(view);
    /// }
    /// ```
    pub fn set_status(&mut self, status: SourceStatus) {
        self.status = status.into();
    }

    /// Copy the complete source status without separating state from reason.
    pub fn status(&self) -> SourceStatus {
        self.status.status()
    }
}

impl std::ops::Deref for SourceOutcome {
    type Target = SourceStatusView;
    fn deref(&self) -> &Self::Target {
        &self.status
    }
}

/// A source notice or request recovery step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum SourceNotice {
    /// Local request limits and source-specific recovery guidance.
    RequestGuidance {
        /// Reviewed guidance without provider response text.
        message: Box<str>,
        /// Seconds left before another source request may be admitted.
        #[serde(skip_serializing_if = "Option::is_none")]
        retry_after_seconds: Option<u64>,
    },
    /// Registered conditions lack the literal words requested in a condition search.
    ConditionWordMismatch {
        /// The source searches several fields and can expand terms; no expansion cause is inferred.
        message: Box<str>,
    },
    /// A checked provider answer survived a local cache failure.
    CacheProblem {
        /// Bounded local failure without paths or database diagnostics.
        reason: CacheProblem,
    },
    /// The source still has graph pages after the bounded fetch window.
    GraphTruncated {
        /// Default-source command offering another graph view.
        fallback: crate::Text,
    },
    /// An empty LitVar lookup can be repeated with a caller-supplied rsID.
    LitvarRsidHint {
        /// Guidance that never supplies or infers an rsID.
        message: Box<str>,
    },
    /// PubMed supplied a named ESearch warning without changing its phrases.
    PubmedWarning {
        /// Original provider warning name, including names added by PubMed later.
        name: String,
        /// Original provider phrases, preserving whitespace and quotes.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        phrases: Box<[String]>,
    },
    /// PubMed did not index these words.
    NotFoundWords {
        /// Whole source words.
        #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
        words: Box<[crate::Text]>,
    },
}

biodata::wire_enum! {
/// A local cache failure that leaves the provider outcome intact.
CacheProblem {
    /// The cache connection or file was busy.
    Busy => "busy",
    /// A SQLite operation failed.
    Database => "database",
    /// Cache JSON could not be encoded or decoded.
    Encoding => "encoding",
    /// A cache filesystem operation failed.
    Filesystem => "filesystem",
    /// Cache state or synchronization was inconsistent.
    Internal => "internal",
    /// Cache content exceeded a supported bound or failed validation.
    Limit => "limit",
    /// A stored snapshot could not be encoded or decoded.
    Snapshot => "snapshot",
    /// Cache ownership or schema validation failed.
    Version => "version",
}
}
