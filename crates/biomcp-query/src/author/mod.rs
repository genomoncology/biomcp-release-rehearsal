//! Provider-record author search and independent exact cards.
mod get;
mod literature;
mod next;
mod orcid;
mod papers;
pub(crate) use papers::papers;
mod search;
mod semantic_scholar;
use crate::next::run;
pub(super) use crate::status::empty;
use crate::{Action, Outcome};
use biomcp_types::{Problem, ProblemCode, Reason, Request, Source, SourceState, StepKind};
pub(crate) use get::get;
pub(crate) use search::search;
fn recovery(cx: &crate::Ctx<'_>, out: &mut Outcome, request: Request) {
    retry(out, request);
    key_recovery(cx, out);
}
fn retry(out: &mut Outcome, request: Request) {
    for source in &out.sources {
        if source.reason == Some(Reason::BadReply) {
            out.problems.push(Problem {
                source: source.source,
                record: None,
                field: "author".into(),
                code: ProblemCode::ValueUnknown,
            });
        }
    }
    if out
        .sources
        .iter()
        .any(|s| matches!(s.state, SourceState::Failed | SourceState::Stale))
    {
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &out.sources
                .iter()
                .map(|source| source.source)
                .collect::<Vec<_>>(),
            request,
            true,
        ));
    }
}
fn key_recovery(cx: &crate::Ctx<'_>, out: &mut Outcome) {
    if out
        .sources
        .iter()
        .any(|s| s.source == Source::SemanticScholar && s.reason == Some(Reason::RateLimited))
        && !cx
            .fetch
            .has_key(biomcp_fetch::SecretName::SemanticScholarApiKey)
    {
        key_step(out);
    }
}
fn key_step(out: &mut Outcome) {
    if !out.next.iter().any(|step| step.0 == StepKind::Key) {
        out.next.push(crate::next::Step::new(
            StepKind::Key,
            &[Source::SemanticScholar],
            Action::KeyHelp(biomcp_fetch::SecretName::SemanticScholarApiKey),
        ));
    }
}

fn source_error(error: Option<biomcp_fetch::RequestError>) -> Option<biomcp_fetch::RequestError> {
    error.filter(|error| !matches!(error, biomcp_fetch::RequestError::Source(error) if error.reason == Reason::NotFound))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NativePages<P> {
    pages: Vec<P>,
    source: biomcp_types::SourceOutcome,
    key_needed: bool,
}

impl crate::source_snapshot::Observation
    for NativePages<(biomcp_adapters::semantic_scholar::author::Page, bool)>
{
    fn whole(&self) -> bool {
        self.pages.iter().all(|(_, refused)| !refused)
            && crate::source_snapshot::sources_whole(std::slice::from_ref(&self.source))
    }
}
impl crate::source_snapshot::Observation
    for NativePages<biomcp_adapters::semantic_scholar::author::PapersPage>
{
    fn whole(&self) -> bool {
        self.pages
            .iter()
            .all(|page| page.refusals == biomcp_adapters::Refusal::None)
            && crate::source_snapshot::sources_whole(std::slice::from_ref(&self.source))
    }
}
