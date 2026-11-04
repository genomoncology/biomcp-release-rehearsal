//! Checked follow-ups, source restrictions and response navigation policy.
use crate::{Action, Outcome};
use biomcp_types::{ReplyData, Source, StepKind};

/// One typed follow-up and the sources it would contact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step(
    /// Follow-up role.
    pub StepKind,
    /// Checked action to spell.
    pub Action,
    /// Native providers required by this follow-up.
    pub Box<[Source]>,
);
impl Step {
    /// Retain target identities supplied by the entity that creates the step.
    pub(crate) fn new(kind: StepKind, sources: &[Source], action: Action) -> Self {
        Self(kind, action, sources.into())
    }
    /// Retain a checked query and its cache policy with the declared provider requirements.
    pub(crate) fn run(
        kind: StepKind,
        sources: &[Source],
        request: biomcp_types::Request,
        fresh: bool,
    ) -> Self {
        Self::new(kind, sources, Action::Run { request, fresh })
    }
    fn is_get(&self) -> bool {
        use biomcp_types::Request::*;
        matches!(
            &self.1,
            Action::Run {
                request: GetGene(_)
                    | GetVariant(_)
                    | GetArticle(_)
                    | GetTrial(_)
                    | GetDisease(_)
                    | GetDrug(_)
                    | GetWhoMedicine(_)
                    | GetAdverseEvent(_)
                    | GetDiagnostic(_)
                    | GetPgx(_)
                    | GetPgxAnnotation(_)
                    | GetStudy(_)
                    | GetProtein(_)
                    | GetPathway(_)
                    | GetPhenotype(_)
                    | GetBiomarker(_)
                    | GetAuthor(_)
                    | GetCellLine(_)
                    | GetGwasAssociation(_)
                    | GetGwasStudy(_),
                ..
            }
        )
    }
}

/// Wrap an already checked query without changing its cache policy.
pub(crate) fn run(request: biomcp_types::Request) -> Action {
    Action::Run {
        request,
        fresh: false,
    }
}

/// Keep failed construction visible without exposing rejected field values.
pub(crate) fn checked<T, E>(
    result: Result<T, E>,
    source: Source,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(_) => {
            problems.push(biomcp_types::Problem {
                source,
                record: None,
                field: "next".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
            None
        }
    }
}

/// Apply source exclusions and the shared response ordering and caps.
pub(crate) fn finish(fetch: &biomcp_fetch::Fetch, result: &mut Outcome) {
    result.next.retain(|step| {
        !matches!(step.1, Action::NetworkHelp(_))
            && !step.2.iter().any(|source| fetch.disabled(*source))
    });
    let has_more = result
        .page
        .as_ref()
        .is_some_and(|page| u64::from(page.offset) + u64::from(page.returned) < page.total);
    match navigation(&result.body) {
        Navigation::NativeOrder => append_network_steps(&mut result.next, &result.sources),
        Navigation::BoundedPma => {
            append_network_steps(&mut result.next, &result.sources);
            result.next.truncate(5);
        }
        Navigation::Bounded => network_steps(&mut result.next, &result.sources, None),
        Navigation::Selectable => network_steps(&mut result.next, &result.sources, Some(has_more)),
        Navigation::Nested(page) => network_steps(
            &mut result.next,
            &result.sources,
            Some(has_more || u64::from(page.offset) + u64::from(page.returned) < page.total),
        ),
    }
}

#[derive(Debug)]
enum Navigation<'a> {
    NativeOrder,
    Bounded,
    Selectable,
    Nested(&'a biomcp_types::PgxPageInfo),
    // PMA cards retain section-first order even when they contain a pivot.
    BoundedPma,
}

fn navigation(body: &biomcp_types::ReplyData) -> Navigation<'_> {
    match body {
        ReplyData::BiomarkerPage { .. }
        | ReplyData::BiomarkerResult { .. }
        | ReplyData::OrganizationPage { .. }
        | ReplyData::AuthorPage { .. }
        | ReplyData::CellLinePage { .. }
        | ReplyData::CellLineFilteredPage { .. } => Navigation::NativeOrder,
        ReplyData::AuthorCards { .. } => Navigation::BoundedPma,
        ReplyData::CellLineChembl { .. } | ReplyData::CellLineAssayCount { .. } => {
            Navigation::BoundedPma
        }
        ReplyData::CellLineCard { .. } => Navigation::BoundedPma,
        ReplyData::GwasAssociationCard { .. }
        | ReplyData::GwasStudy { .. }
        | ReplyData::GwasLoci { .. }
        | ReplyData::GwasAncestries { .. }
        | ReplyData::GwasStudyPage { .. }
        | ReplyData::GwasPage { .. }
        | ReplyData::PathwayEvents { .. }
        | ReplyData::ProteinComplexes { .. } => Navigation::NativeOrder,
        ReplyData::PathwayCard { body: card, .. } => pathway_navigation(card.as_ref()),
        ReplyData::ProteinPage { .. } => Navigation::Selectable,
        ReplyData::ProteinFeatures { .. }
        | ReplyData::ProteinIsoforms { .. }
        | ReplyData::ProteinDomains { .. }
        | ReplyData::ProteinComplexMembers { .. }
        | ReplyData::ProteinStructures { .. }
        | ReplyData::ProteinSequenceResult { .. } => Navigation::Bounded,
        ReplyData::ProteinCard {
            body: Some(card), ..
        } => protein_navigation(card),
        ReplyData::StudyProfiles { .. }
        | ReplyData::StudySampleLists { .. }
        | ReplyData::StudySamples { .. }
        | ReplyData::StudyMutations { .. }
        | ReplyData::StudyFiles { .. } => Navigation::Bounded,
        ReplyData::PathwayPage { .. } | ReplyData::PhenotypePage { .. } => Navigation::Selectable,
        ReplyData::PhenotypeTermCard { .. }
        | ReplyData::PhenotypeSynonyms { .. }
        | ReplyData::PhenotypeXrefs { .. }
        | ReplyData::PhenotypeHierarchy { .. } => Navigation::Bounded,
        ReplyData::PathwayGenes { .. } => Navigation::Bounded,
        ReplyData::PgxAnnotations {
            body: Some(value), ..
        } => pgx_navigation(value),
        ReplyData::VariantCard {
            body: Some(card), ..
        } => variant_navigation(card),
        ReplyData::DiagnosticCard {
            body: Some(biomcp_types::DiagnosticCard::PmaSubmission(_)),
            ..
        } => Navigation::BoundedPma,
        _ => bounded_navigation(body),
    }
}

fn candidate_navigation(candidates: bool) -> Navigation<'static> {
    if candidates {
        Navigation::Selectable
    } else {
        Navigation::Bounded
    }
}

/// Build source network guidance for housekeeping results with the same bounded policy.
pub(crate) fn network(sources: &[biomcp_types::SourceOutcome]) -> Vec<Step> {
    let mut steps = Vec::new();
    network_steps(&mut steps, sources, None);
    steps
}

fn network_steps(
    steps: &mut Vec<Step>,
    sources: &[biomcp_types::SourceOutcome],
    candidate_page: Option<bool>,
) {
    let association = steps
        .iter()
        .any(|step| step.0 == biomcp_types::StepKind::Pivot);
    if candidate_page.is_none() && !association {
        append_network_steps(steps, sources);
        steps.truncate(5);
        return;
    }
    // Some disambiguate actions are searches or relations, not candidate gets.
    let (mut candidates, mut extras): (Vec<_>, Vec<_>) =
        std::mem::take(steps).into_iter().partition(|step| {
            step.0 == biomcp_types::StepKind::Pivot
                || step.0 == biomcp_types::StepKind::Disambiguate && step.is_get()
        });
    if candidates.is_empty() {
        append_network_steps(&mut extras, sources);
        extras.truncate(5);
        *steps = extras;
        return;
    }
    // A section may itself have page actions. Only the response page is protected.
    if (candidate_page == Some(true) || association)
        && let Some(index) = extras
            .iter()
            .position(|step| step.0 == biomcp_types::StepKind::Page)
    {
        candidates.push(extras.remove(index));
    }
    append_network_steps(&mut extras, sources);
    extras.truncate(3);
    candidates.extend(extras);
    *steps = candidates;
}
fn append_network_steps(steps: &mut Vec<Step>, sources: &[biomcp_types::SourceOutcome]) {
    for (reason, setting) in [
        (biomcp_types::Reason::Tls, "ca_bundle"),
        (biomcp_types::Reason::Proxy, "proxy"),
    ] {
        if sources.iter().any(|source| source.reason == Some(reason)) {
            steps.insert(
                0,
                Step::new(
                    biomcp_types::StepKind::Configure,
                    &[],
                    Action::NetworkHelp(setting.into()),
                ),
            );
        }
    }
}

fn pathway_navigation(card: Option<&biomcp_types::PathwayCard>) -> Navigation<'_> {
    if card.is_some_and(|card| card.events.is_some() && card.genes.is_none()) {
        Navigation::NativeOrder
    } else {
        Navigation::Bounded
    }
}
fn protein_navigation(card: &biomcp_types::ProteinCard) -> Navigation<'_> {
    if card.items.iter().any(|item| {
        item.sections
            .as_ref()
            .is_some_and(|s| s.domains.is_some() || s.complexes.is_some())
    }) {
        return Navigation::NativeOrder;
    }
    candidate_navigation(card.items.iter().any(|item| !item.candidates.is_empty()))
}

fn variant_navigation(card: &biomcp_types::VariantCard) -> Navigation<'static> {
    candidate_navigation(card.resolution == biomcp_types::VariantResolution::Candidates)
}

fn pgx_navigation(value: &biomcp_types::PgxAnnotations) -> Navigation<'_> {
    match value {
        biomcp_types::PgxAnnotations::Clinical { page, .. } => Navigation::Nested(page),
        biomcp_types::PgxAnnotations::Guideline { .. }
        | biomcp_types::PgxAnnotations::Label { .. } => Navigation::Bounded,
    }
}

fn bounded_navigation(body: &biomcp_types::ReplyData) -> Navigation<'_> {
    match body {
        ReplyData::StudyPage { .. }
        | ReplyData::DiagnosticCompanion { .. }
        | ReplyData::DiagnosticPage { .. }
        | ReplyData::DiagnosticGenes { .. }
        | ReplyData::DiagnosticSupplements { .. }
        | ReplyData::AdverseEventPage { .. }
        | ReplyData::DiseasePage { .. }
        | ReplyData::PhenotypeDiseases { .. }
        | ReplyData::PhenotypeGenes { .. }
        | ReplyData::DiseaseGenes { .. }
        | ReplyData::DiseasePhenotypes { .. }
        | ReplyData::DrugPage { .. }
        | ReplyData::DrugWhoPage { .. }
        | ReplyData::DrugRecallCard { .. }
        | ReplyData::DrugRecallPage { .. }
        | ReplyData::DrugFdaPage { .. }
        | ReplyData::DrugCivic { .. }
        | ReplyData::VariantPage { .. } => Navigation::Selectable,
        _ => Navigation::Bounded,
    }
}
