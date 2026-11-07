//! Machine-written summaries retain their independent counts and source model.
use crate::mapped::Problems;
use crate::{native_text::NativeText, raw::Field};
use biomcp_fetch::{BadReply, Reply};
use biomcp_types::Tldr;
use serde::Deserialize;

/// Independent machine-written summary and source counts.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaperSummary {
    // A successful HTTP status can still carry the provider's protocol error.
    error: Option<serde::de::IgnoredAny>,
    /// Null means the source has no summary.
    tldr: Option<Field<Summary>>,
    /// Independently reported citation count.
    citation_count: Option<Field<u64>>,
    /// Independently reported influential citation count.
    influential_citation_count: Option<Field<u64>>,
}
/// Source summary and model identifier.
#[derive(Debug, Deserialize)]
struct Summary {
    /// Complete source summary.
    text: Option<Field<NativeText>>,
    /// Source model version.
    model: Option<Field<NativeText>>,
}
impl Reply for PaperSummary {
    fn check(&self) -> Result<(), BadReply> {
        if self.error.is_some() {
            Err(BadReply)
        } else {
            Ok(())
        }
    }
}
impl PaperSummary {
    /// Preserve missing counts independently from an available summary.
    pub fn section(self, problems: &mut Vec<biomcp_types::Problem>) -> biomcp_types::Section<Tldr> {
        let mut p = Problems {
            source: biomcp_types::Source::SemanticScholar,
            record: None,
            list: Vec::new(),
        };
        let cited_by = p.field("cited_by", self.citation_count);
        let influential_cited_by = p.field("influential_cited_by", self.influential_citation_count);
        let value = p.field("tldr", self.tldr).map_or(
            biomcp_types::Section::Empty {
                source: biomcp_types::Source::SemanticScholar,
                record: None,
            },
            |summary| biomcp_types::Section::Data {
                source: biomcp_types::Source::SemanticScholar,
                data: Tldr {
                    text: p.source_field("tldr.text", summary.text),
                    model: p.source_field("tldr.model", summary.model),
                    machine_written: true,
                    cited_by,
                    influential_cited_by,
                },
            },
        );
        problems.extend(p.list);
        value
    }
}
