#![deny(clippy::wildcard_enum_match_arm)]
use crate::{Ctx, gene};

use crate::Outcome;
pub use biomcp_adapters::decl::Bundle;
use biomcp_types::Request;
/// Typed action before an interface spells command arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Explain a network configuration setting.
    NetworkHelp(Box<str>),
    /// Offer a PubMed search with its source explicit in the caller command.
    PubmedSearch(biomcp_types::ArticleSearch),
    /// Follow an exact checked author record to its deferred native paper surface.
    AuthorPapers(biomcp_types::AuthorPapersQuery),
    /// Remove expired local state and older incompatible cache files.
    StoreClean,
    /// Probe source reachability and report publisher terms.
    Health,
    /// Run a query, retaining fresh policy.
    Run {
        #[doc = "Checked query."]
        request: Request,
        #[doc = "Bypass cache reads."]
        fresh: bool,
    },
    /// Explain configuration names for a keyed source.
    KeyHelp(biomcp_fetch::SecretName),
    /// Explain the required arguments for an existing query.
    QueryHelp(Request),
    /// Synchronize one declared file.
    Sync {
        #[doc = "Bundle selector."]
        bundle: Bundle,
    },
}
/// Run one checked request through the shared query layer.
pub fn run(cx: &Ctx<'_>, request: &Request) -> Outcome {
    let mut out = match request {
        Request::AuthorPapers(q) => crate::author::papers(cx, q),
        Request::SearchAuthor(q) => crate::author::search(cx, q),
        Request::GetAuthor(q) => crate::author::get(cx, q),
        Request::CellLineAssayCount(q) => crate::cell_line::assay_count(cx, q),
        Request::SearchCellLine(q) => crate::cell_line::search(cx, q),
        Request::GetCellLine(q) => crate::cell_line::get(cx, q),
        Request::GetGwasAssociation(q) => crate::gwas::get(cx, &q.clone().into()),
        Request::GetGwasStudy(q) => crate::gwas::get(cx, &q.clone().into()),
        Request::SearchGwas(q) => crate::gwas::search(cx, q),
        Request::ProteinDomains(q) => crate::protein::domains(cx, q),
        Request::ProteinComplexes(q) => crate::protein::complexes(cx, q),
        Request::ProteinComplexMembers(q) => crate::protein::complex_members(cx, q),
        Request::ProteinFeatures(q) => crate::protein::features(cx, q),
        Request::ProteinIsoforms(q) => crate::protein::isoforms(cx, q),
        Request::ProteinStructures(q) => crate::protein::structures(cx, q),
        Request::ProteinSequence(q) => crate::protein::sequence(cx, q),
        Request::SearchProtein(q) => crate::protein::search(cx, q),
        Request::GetProtein(q) => crate::protein::get(cx, q),
        Request::SearchPhenotype(q) => crate::phenotype::search(cx, q),
        Request::GetPhenotype(q) => crate::phenotype::get(cx, q),
        Request::SearchBiomarker(q) => crate::biomarker::search(cx, q),
        Request::GetBiomarker(q) => crate::biomarker::get(cx, q),
        Request::SearchOrganization(q) => crate::organization::search(cx, q),
        Request::SearchPathway(q) => crate::pathway::search(cx, q),
        Request::GetPathway(q) => crate::pathway::get(cx, q),
        Request::PathwayGenes(q) => crate::pathway::genes(cx, q),
        Request::PathwayEvents(q) => crate::pathway::events(cx, q),
        Request::SearchStudy(q) => crate::study::search(cx, q),
        Request::GetStudy(q) => crate::study::get(cx, q),
        Request::SearchPgx(query) => crate::pgx::search(cx, query),
        Request::GetPgx(query) => crate::pgx::get(cx, query),
        Request::GetPgxAnnotation(query) => crate::pgx::annotation(cx, query),
        Request::SearchDiagnostic(q) => crate::diagnostic::search(cx, q),
        Request::GetDiagnostic(q) => crate::diagnostic::get(cx, q),
        Request::SearchAdverseEvent(query) => crate::adverse_event::search_command(cx, query),
        Request::GetAdverseEvent(q) if q.source() == biomcp_types::AdverseEventSource::Maude => {
            crate::adverse_event_maude::get(cx, q)
        }
        Request::GetAdverseEvent(q) => crate::adverse_event::get(cx, q),
        Request::SearchDisease(query) => crate::disease::search(cx, query),
        Request::GetDisease(query) => crate::disease::get(cx, query),
        Request::SearchDrug(query) => crate::drug::search_with_aliases(cx, query),
        Request::GetDrug(query) => crate::drug::get(cx, query),
        Request::GetWhoMedicine(query) => crate::drug::who::get(cx, query),
        Request::VariantAssertions(query) => crate::variant::assertions::get(cx, query),
        Request::VariantPopulation(query) => crate::variant::population_detail::get(cx, query),
        Request::VariantNormalize(query) => crate::variant::normalize::normalize(cx, query),
        Request::SearchVariant(query) => crate::variant::search::search(cx, query),
        Request::VariantRecurrence(query) => crate::variant::recurrence::run(cx, query),
        Request::GetVariant(query) => crate::variant::get(cx, query),
        Request::TrialDocument(query) => crate::trial::document::download(cx, query, false).out,
        Request::GetTrial(query) => crate::trial::get::get(cx, query),
        Request::SearchTrial(query) => crate::trial::search(cx, query),
        Request::ArticlePdf(query) => crate::article::pdf::download(cx, query, false).out,
        Request::ArticleEvidence(query) => crate::article::evidence::run(cx, query),
        Request::ArticleRecommend(query) => crate::article::recommend::run(cx, query),
        Request::ArticleSupplement(query) => crate::article::supplement::read(cx, query),
        Request::ArticleText(query) => crate::article::text::read(cx, query),
        Request::ArticleCitations(query) => crate::article::edges(cx, query, false),
        Request::ArticleReferences(query) => crate::article::edges(cx, query, true),
        Request::GetArticle(query) => crate::article::get::get(cx, query),
        Request::SearchArticle(query) => crate::article::search(cx, query),
        Request::GetGene(query) => gene::get::get(cx, query),
        Request::SearchGene(query) => gene::search::search(cx, query),
    };
    crate::publish::finish(&mut out, request);
    out
}
