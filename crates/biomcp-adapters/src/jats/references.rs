//! Preserve explicit source reference identities for later joins; perform no citation lookup.
use super::blocks;
use biodata::{Doi, Pmid, Text};

use biomcp_fetch::BadReply;
/// One JATS reference's own checked identifiers.
#[derive(Debug)]
pub struct Reference {
    /// Source reference identity, matched only to explicit xref/rid values.
    pub id: Text,
    /// Explicit PubMed identity, when canonical.
    pub pmid: Option<Pmid>,
    /// Explicit DOI, when canonical.
    pub doi: Option<Doi>,
}
pub(super) fn rows(
    xml: &str,
    problems: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> Result<Vec<Reference>, BadReply> {
    let mut out = Vec::new();
    for reference in blocks::descendants(xml, "ref")? {
        let Some(id) = reference.attr("id")? else {
            *refusals = crate::Refusal::Refused;
            continue;
        };
        let Some(id) = problems.optional("references.id", Some(Text::new(&id))) else {
            *refusals = crate::Refusal::Refused;
            continue;
        };
        let mut row = Reference {
            id,
            pmid: None,
            doi: None,
        };
        for node in blocks::descendants(reference.0, "pub-id")? {
            let value = node.text()?;
            match node.attr("pub-id-type")?.as_deref() {
                Some("pmid") => {
                    row.pmid = problems.optional("references.pmid", Some(value.parse()))
                }
                Some("doi") => row.doi = problems.optional("references.doi", Some(value.parse())),
                _ => {}
            }
        }
        out.push(row);
    }
    Ok(out)
}
pub(super) fn ids(xml: &str) -> Result<Box<[Text]>, BadReply> {
    typed_ids(xml, &["bibr", "ref"])
}
pub(super) fn table_ids(xml: &str) -> Result<Box<[Text]>, BadReply> {
    typed_ids(xml, &["table", "table-wrap"])
}
fn typed_ids(xml: &str, kinds: &[&str]) -> Result<Box<[Text]>, BadReply> {
    let mut out = Vec::new();
    for node in blocks::descendants(xml, "xref")? {
        if !node
            .attr("ref-type")?
            .is_some_and(|kind| kinds.contains(&kind.as_str()))
        {
            continue;
        }
        if let Some(ids) = node.attr("rid")? {
            for id in ids.split_whitespace() {
                let id = Text::new(id).map_err(|_| BadReply)?;
                if !out.contains(&id) {
                    out.push(id);
                }
            }
        }
    }
    Ok(out.into())
}

/// One original JATS paragraph and its explicit reference relationships.
#[derive(Clone, Debug)]
pub struct Paragraph {
    /// Whole original paragraph as Markdown.
    pub markdown: String,
    /// Exact source xref/rid identifiers.
    pub references: Box<[Text]>,
}
pub(super) fn paragraphs(xml: &str) -> Result<Vec<Paragraph>, BadReply> {
    blocks::descendants(xml, "p")?
        .into_iter()
        .map(|node| {
            Ok(Paragraph {
                markdown: super::markdown::inline(node.0)?.trim().to_owned(),
                references: ids(node.0)?,
            })
        })
        .collect()
}
