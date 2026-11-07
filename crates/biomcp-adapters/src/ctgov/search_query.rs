//! Registry-owned name broadening follows a checked empty exact first page.
use super as ctgov;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{SourceNotice, TrialSearch};

/// Most words the registry accepts in one quoted name, measured live: a longer name fails the
/// whole request with 400 "Too complicated query", whichever other names are beside it.
const NAME_WORDS: usize = 12;

/// Whether the registry can search this name, counting every run of letters or digits as a word.
pub fn name_fits(name: &str) -> bool {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .count()
        <= NAME_WORDS
}

/// Search typed selectors with the registry name fallback.
pub fn search_query(
    leg: &Leg<'_>,
    query: &TrialSearch,
    gene: Option<&str>,
    conditions: &[biodata::Text],
) -> (
    Result<ctgov::SearchPage, RequestError>,
    Option<SourceNotice>,
) {
    let mut params = ctgov::parameters(query, gene);
    if query.condition_id().is_some()
        && let Some(condition) = conditions.first()
    {
        params.insert("query.cond".into(), format!("({condition})"));
    }
    let context_gene = gene.filter(|_| !query.count_only());
    let mut result =
        ctgov::search_with_gene(leg, &params, conditions, query.country(), context_gene);
    let mut broader = query.broader_names();
    let notice = if query.next_page().is_none()
        && (query.sponsor().is_some() || query.facility().is_some())
        && result.as_ref().is_ok_and(|page| page.total == Some(0))
    {
        // Replace only the exact name selectors. Keep resolved condition/gene
        // parameters and every other strict filter from the actual first request.
        let mut fallback = ctgov::broader_parameters(query, None);
        for (name, value) in &params {
            if name != "filter.advanced" {
                fallback.insert(name.clone(), value.clone());
            }
        }
        let message = format!(
            "ClinicalTrials.gov exact phrase request {params:?} returned zero trials. \
             BioMCP tried the registry's broader name request {fallback:?} once."
        );
        broader = true;
        result = ctgov::search_with_gene(leg, &fallback, conditions, query.country(), context_gene);
        Some(guidance(message))
    } else if broader {
        Some(guidance(format!(
            "ClinicalTrials.gov broader name search continuation request {params:?}."
        )))
    } else {
        None
    };
    if broader {
        result = result.and_then(wrap_page);
    }
    (result, notice)
}

fn wrap_page(mut page: ctgov::SearchPage) -> Result<ctgov::SearchPage, RequestError> {
    page.next = page
        .next
        .as_ref()
        .map(TrialSearch::broader_page_token)
        .transpose()
        .map_err(|_| {
            RequestError::from(biomcp_fetch::SourceError {
                source: ctgov::SPEC.source,
                reason: biomcp_types::Reason::BadReply,
                status: Some(200),
                tls_cause: None,
                retry_after: None,
            })
        })?;
    Ok(page)
}

fn guidance(message: String) -> SourceNotice {
    SourceNotice::RequestGuidance {
        message: message.into(),
        retry_after_seconds: None,
    }
}
