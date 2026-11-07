//! Independent citation graph and explicit JATS reference evidence with stored continuations.
use biomcp_types::ReplyBody;
mod graph;
mod locator;
mod page;
mod text;
use crate::{Action, Ctx, Outcome, status, store::Store};

use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::{
    ArticleEvidence, ArticleEvidenceResult, CitationContexts, CitationParagraphs, ReplyData,
    Request, Section, Source, SourceOutcome, Status, StepKind,
};
use sha2::{Digest, Sha256};
// Source stages leave a bounded part of the same command budget for publishing
// successful independent evidence and its continuation after a source timeout.
const SAVE_BUDGET: std::time::Duration = std::time::Duration::from_millis(100);
pub(super) struct Snapshot {
    dropped: bool,
    graph: Section<CitationContexts>,
    target: Option<graph::Target>,
    fulltext: Option<Section<CitationParagraphs>>,
    terminal: bool,
    cursor: Option<u32>,
    matched: bool,
    sources: Vec<SourceOutcome>,
}
impl Snapshot {
    fn empty() -> Self {
        Self {
            dropped: false,
            graph: Section::Empty {
                source: Source::SemanticScholar,
                record: None,
            },
            target: None,
            fulltext: None,
            terminal: false,
            cursor: None,
            matched: false,
            sources: Vec::new(),
        }
    }
}
/// Keep every successful evidence part beside any independent source failure.
pub(crate) fn run(cx: &Ctx<'_>, query: &ArticleEvidence) -> Outcome {
    let mut out = super::empty(
        (ArticleEvidenceResult {
            semantic_scholar: Section::Empty {
                source: Source::SemanticScholar,
                record: None,
            },
            fulltext: query.fulltext().then_some(Section::NotApplicable),
        })
        .with_query(query.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    if let Err(error) = execute(cx, query, &mut deadline, &mut out) {
        out.local = status::local(error);
        crate::status::item(&mut out, false, true, false);
    }
    command.finish(deadline.remaining());
    if out
        .sources
        .iter()
        .any(|s| s.reason == Some(biomcp_types::Reason::RateLimited))
    {
        out.next.insert(
            0,
            crate::next::Step::new(
                StepKind::Key,
                &[biomcp_types::Source::SemanticScholar],
                Action::KeyHelp(biomcp_fetch::SecretName::SemanticScholarApiKey),
            ),
        );
    }
    if matches!(out.status(), Status::Failed | Status::Partial) && out.next.is_empty() {
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &super::next::evidence_sources(query),
            Request::ArticleEvidence(query.identity()),
            true,
        ));
    }
    out
}
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEvidence,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    // Disabled source content is never restored from a previous continuation snapshot.
    let graph_disabled = cx.fetch.disabled(Source::SemanticScholar);
    let text_disabled = cx.fetch.disabled(Source::PmcOa);
    let mut store = Store::open(cx.fetch.cache_access()?, *deadline)?;
    let identity = serde_json::to_vec(&query.identity()).map_err(|_| CacheError::Encoding)?;
    let key = Sha256::digest([b"biomcp.article_evidence/2\0".as_slice(), &identity].concat());
    let mut restored = false;
    let mut snapshot = if let Some(id) = query.pool() {
        match store.evidence(id.as_str(), &key, |source| cx.fetch.disabled(source))? {
            crate::store::EvidenceState::Missing => {
                out.body = ReplyData::PoolError { mismatch: false };
                restart(query, out);
                return Ok(());
            }
            crate::store::EvidenceState::Disabled => Snapshot::empty(),
            crate::store::EvidenceState::Ready(payload) => {
                restored = true;
                decode(&payload)?.unwrap_or_else(Snapshot::empty)
            }
        }
    } else {
        Snapshot::empty()
    };
    context_rights(cx, query, deadline, &mut snapshot, out);
    if restrict_text(
        &mut snapshot,
        cx.fetch.disabled_use(biomcp_types::Use::PerRecord),
    ) {
        restored = false;
    }
    let mut stages = deadline.child(deadline.remaining().saturating_sub(SAVE_BUDGET));
    let started = stages.remaining();
    if graph_disabled {
        snapshot.graph =
            Section::unavailable(Source::SemanticScholar, biomcp_types::Reason::Disabled);
        snapshot.terminal = true;
        snapshot
            .sources
            .retain(|s| s.source != Source::SemanticScholar);
    } else if !snapshot.terminal {
        graph::read(cx, query, &mut snapshot, &mut stages, out)?;
    }
    if query.fulltext() {
        if text_disabled {
            snapshot.fulltext = Some(Section::unavailable(
                Source::PmcOa,
                biomcp_types::Reason::Disabled,
            ));
            snapshot.sources.retain(|s| s.source != Source::PmcOa);
        } else if snapshot.fulltext.is_none() {
            text::read(cx, query, &mut snapshot, &mut stages, out)?;
        }
    }
    *deadline = super::consume(*deadline, started, stages.remaining());
    store.set_deadline(*deadline);
    // Resolver legs are dependencies of the saved content too. Admit every
    // contributing source before reading any continuation payload next time.
    for incoming in std::mem::take(&mut out.sources) {
        source(&mut snapshot, incoming);
    }
    let payload = {
        snapshot.dropped |= out.dropped;
        encode(&snapshot)?
    };
    let id = crate::publish::keep(
        store
            .save_evidence(
                query.pool().filter(|_| restored),
                &key,
                &payload,
                &snapshot.sources,
            )
            .map_err(Into::into),
        out,
    )?;
    page::render(query, snapshot, id, out)?;
    skipped(out, graph_disabled, query.fulltext() && text_disabled);
    Ok(())
}

fn skipped(out: &mut Outcome, graph: bool, text: bool) {
    // A response observation does not make disabled content a stored dependency.
    for (source, disabled) in [(Source::SemanticScholar, graph), (Source::PmcOa, text)] {
        if disabled && !out.sources.iter().any(|outcome| outcome.source == source) {
            out.sources.push(biomcp_adapters::registry::outcome(
                source,
                biomcp_types::SourceStatus::Skipped(Some(biomcp_types::Reason::Disabled)),
            ));
        }
    }
}
pub(super) fn source(snapshot: &mut Snapshot, incoming: SourceOutcome) {
    if let Some(original) = snapshot
        .sources
        .iter_mut()
        .find(|s| s.source == incoming.source)
    {
        status::retain_source_state(original, &incoming);
    } else {
        snapshot.sources.push(incoming)
    }
}

fn restart(query: &ArticleEvidence, out: &mut Outcome) {
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &super::next::evidence_sources(query),
        Request::ArticleEvidence(query.identity()),
        false,
    ));
}

// Private versioned store layout uses existing checked domain values. The query
// crate does not define a second public wire model or depend on serialization derives.
type Saved = (
    Section<CitationContexts>,
    Option<Section<CitationParagraphs>>,
    (bool, Option<u32>, bool),
    Vec<SourceOutcome>,
    Option<(Option<biodata::Pmid>, Option<biodata::Doi>)>,
    bool,
);
fn encode(snapshot: &Snapshot) -> Result<String, CacheError> {
    serde_json::to_string(&(
        &snapshot.graph,
        &snapshot.fulltext,
        (snapshot.terminal, snapshot.cursor, snapshot.matched),
        &snapshot.sources,
        snapshot
            .target
            .as_ref()
            .map(|target| (target.pmid, target.doi.as_ref())),
        snapshot.dropped,
    ))
    .map_err(|_| CacheError::Snapshot)
}
fn decode(payload: &str) -> Result<Option<Snapshot>, CacheError> {
    let Some((graph, fulltext, (terminal, cursor, matched), sources, target, dropped)): Option<
        Saved,
    > = crate::store::labels::decode(payload).map_err(|_| CacheError::Snapshot)?
    else {
        return Ok(None);
    };
    Ok(Some(Snapshot {
        dropped,
        graph,
        fulltext,
        terminal,
        cursor,
        matched,
        sources,
        target: target.map(|(pmid, doi)| graph::Target { pmid, doi }),
    }))
}

fn context_rights<'a>(
    cx: &Ctx<'a>,
    query: &ArticleEvidence,
    deadline: &mut Deadline<'a>,
    snapshot: &mut Snapshot,
    out: &mut Outcome,
) {
    if !cx.fetch.disabled_use(biomcp_types::Use::PerRecord) {
        return;
    }
    let Section::Data { data, .. } = &mut snapshot.graph else {
        return;
    };
    let mut unchecked = false;
    if data.license.is_none()
        && !data.contexts.items.is_empty()
        && let biomcp_types::ArticleInput::Pmid(id) = query.citing()
    {
        let leg = cx.fetch.leg(&biomcp_adapters::pubmed::SPEC, *deadline);
        let started = leg.remaining();
        let found = biomcp_adapters::article_rights::licenses(&leg, &[*id]);
        data.license = found.get(*id);
        unchecked = found.unchecked(*id);
        *deadline = super::consume(*deadline, started, leg.remaining());
    }
    if !data
        .license
        .is_some_and(biomcp_adapters::pmc_oa::commercial)
        && !data.contexts.items.is_empty()
    {
        out.dropped |= unchecked;
        data.text_withheld = Some(if unchecked {
            biomcp_adapters::article_rights::text_unchecked(query.citing())
        } else {
            biomcp_adapters::article_rights::text_withheld(query.citing())
        });
    }
}
fn restrict_text(snapshot: &mut Snapshot, restricted: bool) -> bool {
    let mut changed = false;
    if restricted
        && let Section::Data { data, .. } = &mut snapshot.graph
        && !data
            .license
            .is_some_and(biomcp_adapters::pmc_oa::commercial)
        && !data.contexts.items.is_empty()
    {
        data.contexts.items = Box::default();
        data.text_withheld
            .get_or_insert(biomcp_types::WithheldArticleText {
                reason: biomcp_types::Reason::Disabled,
                url: None,
            });
        changed = true;
    }
    if restricted
        && snapshot.fulltext.as_ref().is_some_and(|section| {
            matches!(section,
            Section::Data { data, .. } if !biomcp_adapters::pmc_oa::commercial(data.license))
        })
    {
        snapshot.fulltext = Some(Section::unavailable(
            Source::PmcOa,
            biomcp_types::Reason::Disabled,
        ));
        return true;
    }
    changed
}
