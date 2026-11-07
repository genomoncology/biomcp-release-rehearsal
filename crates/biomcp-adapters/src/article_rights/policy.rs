//! One record-owned license decision for article snippets and retained pools.
use super::{Licenses, licenses, semantic};
use biodata::Pmid;
use biomcp_fetch::{BadReply, Leg};
use biomcp_types::{ArticleItem, Use};
use serde_json::Value;

/// A settled refusal kept in LitSense's cached rows.
const WITHHELD: &str = "biomcp_text_withheld";

/// Filter LitSense's original bytes before either raw or mapped persistence.
pub(crate) fn litsense(leg: &Leg<'_>, body: &[u8]) -> Result<Vec<u8>, BadReply> {
    if !leg.disabled_use(Use::PerRecord) {
        return Ok(body.to_vec());
    }
    let mut rows: Value = serde_json::from_slice(body).map_err(|_| BadReply)?;
    let rows_array = rows.as_array_mut().ok_or(BadReply)?;
    let ids: Vec<_> = rows_array
        .iter()
        .filter_map(|row| pmid(&row["pmid"]))
        .collect();
    let found = licenses(leg, &ids);
    litsense_body(rows, &found)
}
fn litsense_body(mut rows: Value, found: &Licenses) -> Result<Vec<u8>, BadReply> {
    for row in rows.as_array_mut().ok_or(BadReply)? {
        let id = pmid(&row["pmid"]);
        let license = id.and_then(|id| found.get(id));
        let refused = row.get(WITHHELD).is_some();
        let unchecked = !refused && id.is_some_and(|id| found.unchecked(id));
        if let Some(row) = row.as_object_mut() {
            // Cache only the native fields this route actually consumes.
            row.retain(|key, _| matches!(key.as_str(), "pmid" | "section" | "text" | WITHHELD));
            if unchecked {
                row.insert("biomcp_text_unchecked".into(), Value::Bool(true));
            }
            if let Some(license) = license {
                row.insert(
                    "biomcp_license".into(),
                    serde_json::to_value(license).map_err(|_| BadReply)?,
                );
            }
        }
        let abstract_text = row["section"].as_str().is_some_and(is_abstract);
        if !permitted(license, abstract_text)
            && let Some(row) = row.as_object_mut()
        {
            row.remove("text");
            row.remove("section");
            if !unchecked {
                row.insert(WITHHELD.into(), Value::Bool(true));
            }
        }
    }
    serde_json::to_vec(&rows).map_err(|_| BadReply)
}

/// Stored article pools repeat the current rule before returning or rewriting rows.
/// Returns whether any row's license is left unchecked, so its pool is not whole.
pub fn items(leg: &Leg<'_>, rows: &mut [ArticleItem]) -> bool {
    if !leg.disabled_use(Use::PerRecord) {
        return false;
    }
    let ids: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.sentence
                .as_ref()
                .is_some_and(|sentence| sentence.license.is_none())
        })
        .filter_map(|row| row.pmid)
        .collect();
    let found = if ids.is_empty() {
        Licenses::default()
    } else {
        licenses(leg, &ids)
    };
    for row in rows.iter_mut() {
        let mut unchecked = false;
        if let Some(sentence) = &mut row.sentence
            && sentence.license.is_none()
            && let Some(id) = row.pmid
        {
            sentence.license = found.get(id);
            unchecked = found.unchecked(id);
        }
        filter_item(row, unchecked);
    }
    rows.iter().any(item_unchecked)
}
/// Whether this row's text rests on a license lookup that did not finish.
pub fn item_unchecked(row: &ArticleItem) -> bool {
    row.text_withheld
        .as_ref()
        .is_some_and(|notice| notice.reason == biomcp_types::Reason::Unavailable)
}
/// Take unchecked papers' sentences out of rows about to be stored. A failed lookup
/// keeps an abstract in the reply, but nothing unsettled reaches the store.
pub fn take_unchecked(
    rows: &mut [ArticleItem],
) -> Vec<(biomcp_types::ArticleKey, biomcp_types::ArticleSentence)> {
    rows.iter_mut()
        .filter(|row| item_unchecked(row))
        .filter_map(|row| Some((row.key.clone()?, row.sentence.take()?)))
        .collect()
}
/// Give this reply's rows the sentences kept out of the store.
pub fn return_unchecked(
    rows: &mut [ArticleItem],
    held: &[(biomcp_types::ArticleKey, biomcp_types::ArticleSentence)],
) {
    for row in rows {
        if let Some((_, sentence)) = held.iter().find(|(key, _)| row.key.as_ref() == Some(key)) {
            row.sentence = Some(sentence.clone());
        }
    }
}
/// Apply the same per-record rule to a checked stored item without provider access.
fn filter_item(row: &mut ArticleItem, unchecked: bool) {
    let license = row.sentence.as_ref().and_then(|sentence| sentence.license);
    let abstract_text = row
        .sentence
        .as_ref()
        .and_then(|sentence| sentence.section.as_ref())
        .is_some_and(|section| is_abstract(section.as_str()));
    let sentence = row.sentence.is_some() && !permitted(license, abstract_text);
    let contexts = row
        .contexts
        .as_ref()
        .is_some_and(|contexts| !contexts.items.is_empty())
        && !permitted(row.context_license, false);
    if sentence {
        row.sentence = None;
    }
    if contexts {
        row.contexts = None;
    }
    if sentence || contexts || unchecked {
        let mut notice = withheld_item(row);
        if unchecked {
            notice.reason = biomcp_types::Reason::Unavailable;
        }
        row.text_withheld = Some(notice);
    } else if row.sentence.is_some() && item_unchecked(row) {
        row.text_withheld = None;
    }
}

pub(crate) fn withheld_item(row: &ArticleItem) -> biomcp_types::WithheldArticleText {
    use biomcp_types::ArticleKey;
    if let Some(id) = row.pmid {
        return withheld(id);
    }
    let url = row.key.as_ref().map(|key| match key {
        ArticleKey::Pubmed(id) => format!("https://pubmed.ncbi.nlm.nih.gov/{id}/"),
        ArticleKey::Pmc(id) => format!("https://pmc.ncbi.nlm.nih.gov/articles/{id}/"),
        ArticleKey::Doi(id) => format!("https://doi.org/{id}"),
        ArticleKey::Europepmc(id) => format!("https://europepmc.org/article/PPR/{id}"),
        ArticleKey::SemanticScholar(id) => format!("https://www.semanticscholar.org/paper/{id}"),
    });
    biomcp_types::WithheldArticleText {
        reason: biomcp_types::Reason::Disabled,
        url: url.and_then(|url| read_url(&url)),
    }
}
pub(crate) fn withheld(id: Pmid) -> biomcp_types::WithheldArticleText {
    text_withheld(&biomcp_types::ArticleInput::Pmid(id))
}
pub(crate) fn unchecked(id: Pmid) -> biomcp_types::WithheldArticleText {
    text_unchecked(&biomcp_types::ArticleInput::Pmid(id))
}
/// Explain text left out because its license could not be checked, which is not a refusal.
pub fn text_unchecked(input: &biomcp_types::ArticleInput) -> biomcp_types::WithheldArticleText {
    biomcp_types::WithheldArticleText {
        reason: biomcp_types::Reason::Unavailable,
        ..text_withheld(input)
    }
}
/// State on the source that a bounded license lookup did not settle every paper.
pub fn unchecked_notice() -> biomcp_types::SourceNotice {
    biomcp_types::SourceNotice::RequestGuidance {
        message: "Some papers' licenses could not be checked within the lookup limit, so their \
                  body text was left out and this reply was not stored. Those papers are marked \
                  `unavailable`; try again or read them at the link given."
            .into(),
        retry_after_seconds: None,
    }
}
/// Explain a text policy refusal and point to the requested paper's public record.
pub fn text_withheld(input: &biomcp_types::ArticleInput) -> biomcp_types::WithheldArticleText {
    use biomcp_types::ArticleInput;
    let url = match input {
        ArticleInput::Pmid(id) => format!("https://pubmed.ncbi.nlm.nih.gov/{id}/"),
        ArticleInput::Pmcid(id) => format!("https://pmc.ncbi.nlm.nih.gov/articles/{id}/"),
        ArticleInput::Doi(id) => format!("https://doi.org/{id}"),
        ArticleInput::EuropePmc(id) => format!("https://europepmc.org/article/PPR/{id}"),
    };
    biomcp_types::WithheldArticleText {
        reason: biomcp_types::Reason::Disabled,
        url: read_url(&url),
    }
}
pub(super) fn pmid(value: &Value) -> Option<Pmid> {
    parse_pmid(
        &value
            .as_str()
            .map(str::to_owned)
            .or_else(|| value.as_u64().map(|id| id.to_string()))?,
    )
}

/// Abstracts retain their source policy when no explicit record license exists.
pub(crate) fn permitted(license: Option<biodata::License>, abstract_text: bool) -> bool {
    if abstract_text {
        !matches!(
            license,
            Some(
                biodata::License::CcByNc
                    | biodata::License::CcByNcSa
                    | biodata::License::CcByNcNd
                    | biodata::License::Tdm
            )
        )
    } else {
        license.is_some_and(crate::pmc_oa::commercial)
    }
}
pub(super) fn is_abstract(section: &str) -> bool {
    matches!(
        section.trim().to_ascii_lowercase().as_str(),
        "title" | "abstract"
    )
}

/// The core record itself supplies Europe PMC's explicit abstract license.
pub(crate) fn europepmc(leg: &Leg<'_>, body: &[u8]) -> Result<Vec<u8>, BadReply> {
    if !leg.disabled_use(Use::PerRecord) {
        return Ok(body.to_vec());
    }
    europepmc_body(body)
}
fn europepmc_body(body: &[u8]) -> Result<Vec<u8>, BadReply> {
    let mut root: Value = serde_json::from_slice(body).map_err(|_| BadReply)?;
    let record = root.get_mut("result").ok_or(BadReply)?;
    let license = record["license"].as_str().map(crate::pmc_oa::license);
    if !permitted(license, true)
        && let Some(record) = record.as_object_mut()
    {
        record.remove("abstractText");
    }
    serde_json::to_vec(&root).map_err(|_| BadReply)
}

/// Repeat abstract rights when reading a source-native author continuation.
pub fn snapshot(leg: &Leg<'_>, body: &[u8]) -> Result<Vec<u8>, BadReply> {
    semantic::snapshot(leg, body)
}

/// An unusable identity cannot establish rights; native mappers retain their field problem.
pub(super) fn parse_pmid(value: &str) -> Option<Pmid> {
    value.parse().ok()
}
fn read_url(value: &str) -> Option<biodata::Text> {
    // URLs are built only from checked article identities; a malformed destination cannot erase a refusal.
    biodata::Text::new(value).ok()
}
