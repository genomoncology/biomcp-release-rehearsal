//! Preserve nullable constraint metrics independently of the requested gene and transcript.
//! Publisher schema: https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/gene.graphql
use crate::raw::Field;
use biodata::{AccessionBase, EnsemblGeneId, FiniteNumber};

use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Genes<const N: usize> {
    pub data: Aliases,
    pub errors: Option<Vec<GraphError>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct GraphError {
    message: String,
}
pub(super) type Aliases = crate::graphql::Aliases<Field<Gene>>;
impl<const N: usize> Reply for Genes<N> {
    fn check(&self) -> Result<(), BadReply> {
        self.data.check::<N>()?;
        let missing = self
            .data
            .0
            .iter()
            .filter(|(_, gene)| gene.is_none())
            .count();
        let errors = self.errors.as_deref().unwrap_or_default();
        if errors.len() != missing || errors.iter().any(|error| error.message != "Gene not found") {
            return Err(BadReply);
        }
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Gene {
    pub gene_id: String,
    pub canonical_transcript_id: Option<Field<String>>,
    pub mane_select_transcript: Option<Field<Mane>>,
    pub gnomad_constraint: Option<Field<Constraint>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Mane {
    pub ensembl_id: String,
}
impl Gene {
    pub(super) fn transcript(&self) -> Result<AccessionBase, &'static str> {
        let (text, field) = if let Some(mane) = &self.mane_select_transcript {
            (
                mane.as_ref().map(|mane| mane.ensembl_id.as_str()),
                "mane_select_transcript.ensembl_id",
            )
        } else {
            (
                self.canonical_transcript_id
                    .as_ref()
                    .and_then(Field::as_ref)
                    .map(String::as_str),
                "canonical_transcript_id",
            )
        };
        let text = text.ok_or(field)?;
        if !text.starts_with("ENST") {
            return Err(field);
        }
        if text.contains('.') {
            text.parse::<biodata::Accession>().map_err(|_| field)?;
        }
        text.split('.')
            .next()
            .ok_or(field)?
            .parse()
            .map_err(|_| field)
    }
}
impl<const N: usize> Genes<N> {
    pub(super) fn eligible(&self, ids: &[EnsemblGeneId]) -> bool {
        ids.iter().enumerate().all(|(index, id)| {
            self.data
                .0
                .iter()
                .find(|(name, _)| *name == format!("g{index}"))
                .is_some_and(|(_, value)| match value {
                    None => true,
                    Some(value) => value.as_ref().is_some_and(|gene| {
                        gene.gene_id == id.base()
                            && (gene.gnomad_constraint.is_none() || gene.transcript().is_ok())
                    }),
                })
        })
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Constraint {
    pub pli: Option<Field<FiniteNumber>>,
    pub oe_lof: Option<Field<FiniteNumber>>,
    pub oe_lof_upper: Option<Field<FiniteNumber>>,
    pub mis_z: Option<Field<FiniteNumber>>,
    pub syn_z: Option<Field<FiniteNumber>>,
    pub flags: Option<Field<Vec<Field<String>>>>,
}
