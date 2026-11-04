use crate::mapped::Problems;
use crate::raw::Field;
use biodata::{GeneSymbol, NcbiGeneId, Text};

use biomcp_fetch::{BadReply, Reply};
use biomcp_types::{Oncokb, OncokbCard, OncokbGeneType, OncokbTranscript};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(transparent)]
pub(super) struct Genes(pub Vec<Gene>);
impl Reply for Genes {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Gene {
    entrez_gene_id: Option<Field<u32>>,
    hugo_symbol: Option<Field<GeneSymbol>>,
    gene_type: Option<Field<String>>,
    highest_sensitive_level: Option<Field<String>>,
    highest_resistance_level: Option<Field<String>>,
    summary: Option<Field<String>>,
    background: Option<Field<String>>,
    grch37_isoform: Option<Field<String>>,
    #[serde(rename = "grch37RefSeq")]
    grch37_refseq: Option<Field<String>>,
    grch38_isoform: Option<Field<String>>,
    #[serde(rename = "grch38RefSeq")]
    grch38_refseq: Option<Field<String>>,
}
impl Gene {
    pub(super) fn mapped(
        self,
        symbol: &GeneSymbol,
        requested: NcbiGeneId,
        problems: &mut Vec<biomcp_types::Problem>,
    ) -> Option<super::GeneData> {
        let mut cx = Problems {
            source: biomcp_types::Source::Oncokb,
            record: None,
            list: std::mem::take(problems),
        };
        let id = cx
            .required(
                "entrezGeneId",
                self.entrez_gene_id
                    .as_ref()
                    .and_then(Field::as_ref)
                    .copied(),
            )
            .and_then(|value| cx.optional("entrezGeneId", Some(NcbiGeneId::new(value))));
        cx.record = id.and_then(crate::mapped::record);
        let native_symbol = cx.required(
            "hugoSymbol",
            self.hugo_symbol.as_ref().and_then(Field::as_ref),
        );
        let accepted = id == Some(requested) && native_symbol == Some(symbol);
        if id.is_some_and(|id| id != requested) {
            cx.required::<()>("entrezGeneId", None);
        }
        if native_symbol.is_some_and(|value| value != symbol) {
            cx.required::<()>("hugoSymbol", None);
        }
        if !accepted {
            *problems = cx.list;
            return None;
        }
        let value = self.values(&mut cx);
        *problems = cx.list;
        Some(value)
    }
    fn values(self, cx: &mut Problems) -> super::GeneData {
        let (gene_type, gene_type_code) = gene_type(self.gene_type, cx);
        let mut text = |value, field| {
            cx.field(field, value)
                .and_then(|value: String| cx.text(field, Some(&value)))
        };
        super::GeneData {
            card: OncokbCard {
                gene_type,
                gene_type_code,
                highest_sensitive_level: text(
                    self.highest_sensitive_level,
                    "highestSensitiveLevel",
                ),
                highest_resistance_level: text(
                    self.highest_resistance_level,
                    "highestResistanceLevel",
                ),
                version: None,
            },
            section: Oncokb {
                version: None,
                summary: text(self.summary, "summary"),
                background: text(self.background, "background"),
                grch37: OncokbTranscript {
                    isoform: text(self.grch37_isoform, "grch37.isoform"),
                    refseq: text(self.grch37_refseq, "grch37.refseq"),
                },
                grch38: OncokbTranscript {
                    isoform: text(self.grch38_isoform, "grch38.isoform"),
                    refseq: text(self.grch38_refseq, "grch38.refseq"),
                },
            },
        }
    }
}

fn gene_type(
    raw: Option<Field<String>>,
    cx: &mut Problems,
) -> (Option<biodata::SourceText>, Option<OncokbGeneType>) {
    let gene_type = cx.field("geneType", raw).and_then(|value| {
        cx.checked_text("geneType", Some(&value))
            .map(|_| biodata::SourceText::from(value))
    });
    let gene_type_code = gene_type.as_ref().and_then(|label| match label.as_str() {
        "ONCOGENE_AND_TSG" => Some(OncokbGeneType::OncogeneAndTsg),
        "ONCOGENE" => Some(OncokbGeneType::Oncogene),
        "TSG" => Some(OncokbGeneType::Tsg),
        "INSUFFICIENT_EVIDENCE" => Some(OncokbGeneType::InsufficientEvidence),
        "NEITHER" => Some(OncokbGeneType::Neither),
        _ => {
            cx.required::<()>("geneType", None);
            None
        }
    });
    (gene_type, gene_type_code)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Info {
    pub data_version: Option<Field<Version>>,
}
#[derive(Deserialize)]
pub(super) struct Version {
    pub version: Option<Field<Text>>,
}
impl Reply for Info {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
