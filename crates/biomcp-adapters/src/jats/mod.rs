//! Source-specific JATS body mapping after the transport's bounded XML check.
mod blocks;
mod supplements;
pub use supplements::Supplement;
mod body;
mod parts;
mod references;
mod render;
const MAX_MARKDOWN_BYTES: usize = 8 * 1024 * 1024;
mod table;
pub use body::BodyRow;
pub use references::{Paragraph, Reference};
mod markdown;
/// Preserve JATS inline markup in a checked Markdown value.
pub fn markdown(xml: &str) -> Result<biodata::Markdown, biomcp_fetch::BadReply> {
    markdown::checked(xml)
}
use crate::xml_element::Element;
use biodata::{Pmcid, Text};

use biomcp_fetch::{BadReply, Reply, xml::XmlReply};
use biomcp_types::FigureCaption;
pub use parts::{Part, Row, TEXT_BYTES, escaped_bytes};
use serde::{Deserialize, Deserializer};
/// Body text with source row boundaries and deterministic reading order.
#[derive(Debug)]
pub struct Body {
    /// Actual refused source bibliography records.
    pub refusals: crate::Refusal,
    /// Source title from article metadata, without an extra lookup.
    pub title: Option<Text>,
    /// Optional values rejected independently of the document.
    pub problems: Vec<biomcp_types::Problem>,
    /// Explicit source article identifiers in metadata order.
    pub identifiers: Vec<(Text, Text)>,
    /// Own PMC identity when present in JATS metadata.
    pub pmcid: Option<Pmcid>,
    /// Bounded body parts.
    pub parts: Vec<Part>,
    /// Whole original paragraphs and table rows, used to find text across fragment boundaries.
    pub rows: Vec<BodyRow>,
    /// Source bibliography identifiers without inferred citation joins.
    pub references: Vec<Reference>,
    /// Figure captions in source order.
    pub figures: Vec<FigureCaption>,
    /// Explicit source supplementary-material declarations.
    pub supplements: Vec<Supplement>,
}
impl<'de> Deserialize<'de> for Body {
    fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(serde::de::Error::custom("JATS requires XML"))
    }
}
impl Reply for Body {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
impl XmlReply for Body {
    fn decode_xml(xml: &str) -> Result<Self, BadReply> {
        let root = blocks::descendants(xml, "article")?
            .into_iter()
            .next()
            .ok_or(BadReply)?;
        let metadata = root.at(&["front", "article-meta"])?;
        let pmcid = own_id(metadata)?;
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::PmcOa,
            record: pmcid.and_then(crate::mapped::record),
            list: Vec::new(),
        };
        let (title, identifiers) = header(metadata, &mut problems)?;
        let body = root.child("body")?.ok_or(BadReply)?;
        let (parts, rows) = render::body(body, &mut problems)?;
        let mut figures = Vec::new();
        for fig in blocks::descendants(body.0, "fig")? {
            let Some(caption) = fig.child("caption")? else {
                continue;
            };
            if caption.text()?.trim().is_empty() {
                continue;
            }
            figures.push(FigureCaption {
                label: problems.optional(
                    "figures.label",
                    Some(Text::from_source(
                        &fig.value(&["label"])?.unwrap_or_else(|| "Figure".into()),
                    )),
                ),
                caption: markdown(caption.0)?,
            });
        }
        let mut refusals = crate::Refusal::None;
        let references = references::rows(root.0, &mut problems, &mut refusals)?;
        Ok(Self {
            refusals,
            title,
            identifiers,
            pmcid,
            supplements: supplements::read(root.0, &mut problems)?,
            parts,
            rows,
            references,
            problems: problems.list,
            figures,
        })
    }
}
fn own_id(metadata: Option<Element<'_>>) -> Result<Option<Pmcid>, BadReply> {
    let Some(metadata) = metadata else {
        return Ok(None);
    };
    let mut own = None;
    for id in metadata.children("article-id")? {
        if matches!(id.attr("pub-id-type")?.as_deref(), Some("pmc" | "pmcid")) {
            let text = id.text()?;
            let text = if text.starts_with("PMC") {
                text
            } else {
                format!("PMC{text}")
            };
            let checked = text.parse().map_err(|_| BadReply)?;
            if own.is_some_and(|v| v != checked) {
                return Err(BadReply);
            }
            own = Some(checked);
        }
    }
    Ok(own)
}

type Header = (Option<Text>, Vec<(Text, Text)>);
fn header(
    metadata: Option<Element<'_>>,
    problems: &mut crate::mapped::Problems,
) -> Result<Header, BadReply> {
    let Some(metadata) = metadata else {
        return Ok((None, Vec::new()));
    };
    let title = metadata
        .value(&["title-group", "article-title"])?
        .and_then(|s| problems.optional("title", Some(Text::from_source(&s))));
    let mut ids = Vec::new();
    for id in metadata.children("article-id")? {
        if let Some(kind) = id.attr("pub-id-type")? {
            let value = id.text()?;
            let kind = problems.optional("identifiers.kind", Some(Text::new(&kind)));
            let value = problems.optional("identifiers.value", Some(Text::new(&value)));
            if let (Some(kind), Some(value)) = (kind, value) {
                ids.push((kind, value));
            }
        }
    }
    Ok((title, ids))
}
