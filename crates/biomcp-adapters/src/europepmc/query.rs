//! Europe PMC field syntax stays beside its sending adapter.
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{ArticleSearch, ArticleType};
/// Search typed selectors and retain the exact provider term for disclosure.
pub fn search_query(
    leg: &Leg<'_>,
    query: &ArticleSearch,
) -> (Result<super::Search, RequestError>, String) {
    let term = term(query);
    (super::search(leg, &term, query.sort()), term)
}
fn term(query: &ArticleSearch) -> String {
    let mut parts = Vec::new();
    if let Some(gene) = query.gene() {
        parts.push(gene.to_string());
    }
    let words = query.words();
    if !words.is_empty() {
        parts.push(words);
    }
    if let Some(author) = query.author() {
        parts.push(format!("AUTH:{}", field(author.as_str())));
    }
    if let Some(journal) = query.journal() {
        parts.push(format!("JOURNAL:{}", field(journal.as_str())));
    }
    let (since, until) = query.dates();
    if since.is_some() || until.is_some() {
        parts.push(format!(
            "FIRST_PDATE:[{} TO {}]",
            since.map_or_else(|| "1800".into(), |v| v.to_string()),
            until.map_or_else(|| "3000".into(), |v| v.to_string())
        ));
    }
    if let Some(kind) = query.kind() {
        let name = match kind {
            ArticleType::Trial => "clinical trial",
            ArticleType::Randomized => "randomized controlled trial",
            ArticleType::Review => "review",
            ArticleType::SystematicReview => "systematic review",
            ArticleType::MetaAnalysis => "meta-analysis",
            ArticleType::Guideline => "guideline",
            ArticleType::CaseReport => "case reports",
        };
        parts.push(format!("PUB_TYPE:\"{name}\""));
    }
    let mut term = parts.join(" AND ");
    if query.open_access() {
        term.push_str(" AND OPEN_ACCESS:y");
    }
    if query.no_preprints() {
        term.push_str(" NOT SRC:PPR");
    }
    if query.exclude_retracted() {
        term.push_str(" NOT PUB_TYPE:\"retracted publication\"");
    }
    term
}

// Structured selectors are literal field values, not additional query syntax.
fn field(value: &str) -> String {
    if value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_'))
    {
        value.to_owned()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}
