//! PubMed wire terms preserve structured selectors and resolved gene names.
use biomcp_fetch::{Leg, RequestError, Secret};
use biomcp_types::{ArticleSearch, ArticleType};
/// Search typed selectors and retain the exact provider term for disclosure.
pub fn search_query(
    leg: &Leg<'_>,
    query: &ArticleSearch,
    names: &[String],
    email: Option<&Secret>,
) -> (Result<super::SearchIds, RequestError>, String) {
    let words = query.words();
    let group = if names.len() > 1 {
        format!("({})", names.join(" OR "))
    } else {
        names.join("")
    };
    let words = if group.is_empty() {
        words
    } else if words.is_empty() {
        group
    } else {
        format!("{group} AND {words}")
    };
    let term = filters(query, words);
    (super::search_ids(leg, &term, query.sort(), email), term)
}
fn filters(query: &ArticleSearch, words: String) -> String {
    let mut parts: Vec<String> = (!words.is_empty()).then_some(words).into_iter().collect();
    if let Some(author) = query.author() {
        parts.push(format!("{author}[au]"));
    }
    if let Some(journal) = query.journal() {
        parts.push(format!("{journal}[ta]"));
    }
    let (since, until) = query.dates();
    if since.is_some() || until.is_some() {
        let date = |value: Option<biodata::PartialDate>, default: &str| {
            value.map_or_else(|| default.to_owned(), |v| v.to_string().replace('-', "/"))
        };
        parts.push(format!(
            "{}:{}[dp]",
            date(since, "1800"),
            date(until, "3000")
        ));
    }
    if let Some(kind) = query.kind() {
        parts.push(
            match kind {
                ArticleType::Trial => "\"clinical trial\"[pt]",
                ArticleType::Randomized => "\"randomized controlled trial\"[pt]",
                ArticleType::Review => "review[pt]",
                ArticleType::SystematicReview => "\"systematic review\"[pt]",
                ArticleType::MetaAnalysis => "\"meta-analysis\"[pt]",
                ArticleType::Guideline => "(guideline[pt] OR \"practice guideline\"[pt])",
                ArticleType::CaseReport => "\"case reports\"[pt]",
            }
            .to_owned(),
        );
    }
    let mut term = parts.join(" AND ");
    if query.open_access() {
        term.push_str(" AND \"pubmed pmc open access\"[filter]");
    }
    if query.no_preprints() {
        term.push_str(" NOT preprint[pt]");
    }
    if query.exclude_retracted() {
        term.push_str(" NOT retracted publication[pt]");
    }
    term
}

/// Resolve an explicit DOI group with native PubMed syntax and request identities.
pub fn search_dois(
    leg: &biomcp_fetch::Leg<'_>,
    batch: &[biodata::Doi],
    email: Option<&biomcp_fetch::Secret>,
) -> Result<super::SearchIds, RequestError> {
    let term = batch
        .iter()
        .map(|doi| {
            format!(
                "\"{}\"[doi]",
                doi.to_string()
                    .to_ascii_lowercase()
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
            )
        })
        .collect::<Vec<_>>()
        .join(" OR ");
    let inputs = Some(biomcp_fetch::Identities {
        namespace: "doi".into(),
        ids: batch.iter().map(ToString::to_string).collect(),
    });
    super::search_ids_with_inputs(leg, &term, biomcp_types::ArticleSort::Rank, email, inputs)
}
