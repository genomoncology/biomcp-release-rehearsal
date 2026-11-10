//! A failed local publication never owns the fetched answer.
use crate::Outcome;
use biomcp_fetch::RequestError;
use biomcp_types::{Paged, Request, SourceNotice, StepKind};

pub(crate) fn keep<T>(
    result: Result<T, RequestError>,
    out: &mut Outcome,
) -> Result<Option<T>, RequestError> {
    match result {
        Ok(saved) => Ok(Some(saved)),
        Err(RequestError::Cache(error)) => {
            out.publish_error = Some(error);
            Ok(None)
        }
        Err(error) => Err(error),
    }
}
pub(crate) fn finish(out: &mut Outcome, request: &Request) {
    let Some(error) = out.publish_error.take() else {
        return;
    };
    let notice = SourceNotice::CacheProblem {
        reason: error.into(),
    };
    for source in &mut out.sources {
        if !source.notices.contains(&notice) {
            source.notices.push(notice.clone());
        }
    }
    if out.sources.is_empty() {
        out.local = Some(error.into());
    }
    out.next.clear();
    out.next.push(crate::next::Step::run(
        StepKind::Fresh,
        &out.sources.iter().map(|s| s.source).collect::<Vec<_>>(),
        restart(request),
        true,
    ));
}
fn restart(request: &Request) -> Request {
    match request {
        Request::SearchArticle(q) => Request::SearchArticle(q.page(0, None)),
        Request::ArticleCitations(q) => Request::ArticleCitations(q.page(0, None)),
        Request::ArticleReferences(q) => Request::ArticleReferences(q.page(0, None)),
        Request::ArticleEvidence(q) => Request::ArticleEvidence(q.identity()),
        Request::ArticleRecommend(q) => Request::ArticleRecommend(q.page(0, None)),
        Request::SearchAuthor(q) => Request::SearchAuthor(q.restart()),
        Request::AuthorPapers(q) => Request::AuthorPapers(q.restart()),
        Request::SearchProtein(q) => Request::SearchProtein(q.restart()),
        Request::SearchGwas(q) => Request::SearchGwas(q.restart()),
        Request::SearchBiomarker(q) => Request::SearchBiomarker(q.restart()),
        Request::SearchOrganization(q) => Request::SearchOrganization(q.restart()),
        Request::SearchStudy(q) => Request::SearchStudy(q.restart()),
        Request::SearchPathway(q) => Request::SearchPathway(q.restart()),
        Request::PathwayGenes(q) => Request::PathwayGenes(q.restart()),
        Request::PathwayEvents(q) => Request::PathwayEvents(q.restart()),
        Request::GetAdverseEvent(q) => Request::GetAdverseEvent(q.restart()),
        Request::SearchAdverseEvent(q) => Request::SearchAdverseEvent(q.restart()),
        Request::GetPhenotype(q) => Request::GetPhenotype(q.restart()),
        Request::GetPgx(q) => Request::GetPgx(q.restart()),
        Request::GetGwasStudy(q) => Request::GetGwasStudy(biomcp_types::GwasStudyGet {
            page: q.page.restart(),
            ..q.clone()
        }),
        Request::GetTrial(q) => Request::GetTrial(q.identity()),
        Request::SearchTrial(q) => Request::SearchTrial(q.first_page()),
        Request::ProteinFeatures(q) => Request::ProteinFeatures(q.restart()),
        Request::ProteinIsoforms(q) => Request::ProteinIsoforms(q.restart()),
        Request::ProteinStructures(q) => Request::ProteinStructures(q.restart()),
        Request::ProteinDomains(q) => Request::ProteinDomains(q.restart()),
        Request::ProteinComplexes(q) => Request::ProteinComplexes(q.restart()),
        Request::ProteinComplexMembers(q) => Request::ProteinComplexMembers(q.restart()),
        _ => request.clone(),
    }
}
