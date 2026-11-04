//! Canonical CA identity is independent of nested representations and references.
//! Allele Registry v0.08.07 pp. 5–7: https://calculator.clinicalgenome.org/redmine/attachments/download/264/AlleleRegistry_0.8.7_api.pdf
//! Every coordinate tuple belongs to one complete definition; never retain a subset as a simpler allele.
use crate::raw::Field;
use biodata::Text;
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct Allele {
    #[serde(rename = "@id")]
    pub id: Option<Field<Text>>,
    #[serde(rename = "errorType")]
    pub error_type: Option<Text>,
    #[serde(rename = "actualAllele")]
    pub actual_allele: Option<Field<Text>>,
    #[serde(rename = "description")]
    pub error_description: Option<Field<biodata::SourceText>>,
    pub message: Option<Field<biodata::SourceText>>,
    #[serde(rename = "genomicAlleles")]
    pub genomic: Option<Field<Vec<Field<Genomic>>>>,
    #[serde(rename = "transcriptAlleles")]
    pub transcripts: Option<Field<Vec<Field<Transcript>>>>,
    #[serde(rename = "externalRecords")]
    pub external: Option<Field<External>>,
}
#[derive(Deserialize)]
pub(super) struct External {
    #[serde(rename = "dbSNP")]
    pub rs: Option<Field<Vec<Field<Rs>>>>,
    #[serde(rename = "ClinVarVariations")]
    pub clinvar: Option<Field<Vec<Field<Clinvar>>>>,
}
#[derive(Deserialize)]
pub(super) struct Rs {
    pub rs: u64,
}
#[derive(Deserialize)]
pub(super) struct Clinvar {
    #[serde(rename = "variationId")]
    pub id: u64,
}
#[derive(Deserialize)]
pub(super) struct Genomic {
    pub chromosome: Option<crate::raw::Field<Text>>,
    #[serde(rename = "referenceGenome")]
    pub assembly: Option<crate::raw::Field<Text>>,
    pub hgvs: Option<Field<Vec<Field<Text>>>>,
    pub coordinates: Option<Field<Vec<Coordinate>>>,
}
#[derive(Deserialize)]
pub(super) struct Coordinate {
    pub start: u32,
    pub end: u32,
    #[serde(rename = "referenceAllele")]
    pub reference: String,
    pub allele: String,
}
impl Reply for Allele {
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if (200..300).contains(&status) || matches!(status, 400 | 422) && self.error_type.is_some()
        {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
    fn check(&self) -> Result<(), BadReply> {
        if let Some(code) = &self.error_type {
            return if self.id.is_none()
                && code.as_str().len() <= 128
                && code
                    .as_str()
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                Ok(())
            } else {
                Err(BadReply)
            };
        }

        if self
            .genomic
            .as_ref()
            .and_then(Field::as_ref)
            .is_some_and(|rows| rows.len() > 1000)
        {
            return Err(BadReply);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub(super) struct Transcript {
    #[serde(rename = "MANE")]
    pub mane: Option<Field<Mane>>,
}
#[derive(Deserialize)]
pub(super) struct Mane {
    pub nucleotide: Option<Field<ManeForms>>,
    pub protein: Option<Field<ManeForms>>,
}
#[derive(Deserialize)]
pub(super) struct ManeForms {
    #[serde(rename = "Ensembl")]
    pub ensembl: Option<Field<Hgvs>>,
    #[serde(rename = "RefSeq")]
    pub refseq: Option<Field<Hgvs>>,
}
#[derive(Deserialize)]
pub(super) struct Hgvs {
    pub hgvs: Field<Text>,
}

impl Allele {
    pub(super) fn identity(
        &self,
        expected: Option<&str>,
    ) -> Result<(Text, Text, biodata::VariantId), BadReply> {
        let url = self.id.as_ref().and_then(Field::as_ref).ok_or(BadReply)?;
        let key = url
            .as_str()
            .strip_prefix("http://reg.genome.network/allele/")
            .ok_or(BadReply)?;
        let id = key.parse::<biodata::VariantId>().map_err(|_| BadReply)?;
        if !matches!(id, biodata::VariantId::Ca(_))
            || expected.is_some_and(|expected| expected != key)
        {
            return Err(BadReply);
        }
        Ok((url.clone(), Text::new(key).map_err(|_| BadReply)?, id))
    }
}
