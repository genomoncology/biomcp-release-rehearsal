//! Checked caller IDs can select one source without making an allele-identity claim.
use crate::{Ctx, Outcome};
use biodata::Variant;
use biomcp_fetch::Deadline;
use biomcp_types::{
    Source, VariantCard, VariantGet, VariantHeader, VariantNamespace, VariantResolution,
    VariantSection,
};
pub(super) fn source(fetch: &biomcp_fetch::Fetch, query: &VariantGet) -> Option<Source> {
    let [section] = query.sections() else {
        return None;
    };
    use VariantSection as S;
    let source = super::sections::source(*section)?;
    if fetch.disabled(source) {
        return Some(source);
    }
    if *section == S::Oncokb
        && (!fetch.has_key(biomcp_fetch::SecretName::OncokbApiKey)
            || query
                .build()
                .is_some_and(|build| super::oncokb::input_genomic(query, build).is_some()))
    {
        return Some(source);
    }
    if *section == S::AlphaGenome && !fetch.has_key(biomcp_fetch::SecretName::AlphaGenomeApiKey) {
        return Some(source);
    }
    match (query.input().namespace(), section) {
        (VariantNamespace::Ca, S::Civic | S::Erepo) | (VariantNamespace::ClinVar, S::Clinvar) => {
            Some(source)
        }
        (VariantNamespace::Rs, S::Gwas) => Some(source),
        _ => None,
    }
}
/// Use the execution router before adding any identity-provider requirement.
pub(crate) fn requirements(fetch: &biomcp_fetch::Fetch, query: &VariantGet) -> Vec<Source> {
    if let Some(source) = source(fetch, query) {
        return vec![source];
    }
    let mut sources = vec![super::source(query).source];
    // Prediction, conservation and COSMIC use the MyVariant component path.
    sources.extend(
        super::sections::selected(query)
            .map(|section| super::sections::source(section).unwrap_or(Source::MyVariant)),
    );
    sources
}

pub(super) fn get(
    cx: &Ctx<'_>,
    query: &VariantGet,
    source: Source,
    mut deadline: Deadline<'_>,
) -> Outcome {
    let requested = query.input().identity().cloned();
    let mut candidates = vec![VariantHeader {
        identity: Variant {
            ids: Box::default(),
            representations: Box::default(),
            genes: Box::default(),
        },
        requested,
        source,
        record: biomcp_types::SourceRecord {
            namespace: biodata::IdentifierNamespace::Unknown,
            id: query.input().text().clone(),
            url: None,
        },
        transcripts: Box::default(),
        myvariant: None,
        sections: None,
        component_terms: Box::default(),
    }];
    let (sources, mut problems, local, incomplete) =
        super::sections::apply(cx, query, &mut candidates, &mut deadline, None);
    let status = status(&sources, incomplete);
    let next = super::next(
        cx.fetch,
        query,
        VariantResolution::Unresolved,
        &candidates,
        None,
        &mut problems,
    );
    crate::status::finish(
        [status],
        crate::status::Parts {
            dropped: false,
            body: super::reply(
                query,
                VariantCard {
                    resolution: VariantResolution::Unresolved,
                    source,
                    candidates: candidates.into(),
                    rejection: None,
                },
                &sources,
            ),
            page: None,
            sources: sources.clone(),
            problems,
            next: {
                let mut next = next;
                next.extend(super::oncokb::next(query, &sources));
                next.extend(super::alphagenome::next(&sources));
                next
            },
            local,
        },
    )
}

fn status(sources: &[biomcp_types::SourceOutcome], incomplete: bool) -> biomcp_types::Status {
    if sources.is_empty()
        || sources
            .iter()
            .all(|s| s.state == biomcp_types::SourceState::Failed)
    {
        crate::status::item_status(false, true, false)
    } else if sources.iter().any(|s| {
        matches!(
            s.state,
            biomcp_types::SourceState::Failed | biomcp_types::SourceState::Stale
        )
    }) || incomplete
    {
        crate::status::item_status(true, false, true)
    } else {
        crate::status::item_status(true, false, false)
    }
}
