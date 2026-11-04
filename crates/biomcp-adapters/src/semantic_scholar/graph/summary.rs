use super::PaperSummary;
use crate::mapped::Problems;
use biomcp_types::Tldr;

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
