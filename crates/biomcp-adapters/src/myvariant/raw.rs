use crate::raw::{Field, OneOrMany};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Deserialize)]
pub(super) struct Page {
    pub total: u64,
    pub hits: Vec<Field<Hit>>,
}
#[derive(Deserialize)]
pub(super) struct Annotation {
    #[serde(flatten)]
    pub hit: Field<Hit>,
}
impl Reply for Annotation {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
pub(super) struct Hit {
    #[serde(rename = "_id")]
    pub id: biodata::Text,
    pub dbsnp: Option<Field<Dbsnp>>,
    pub snpeff: Option<crate::raw::Field<super::transcripts::Snpeff>>,
    pub clinvar: Option<Field<Clinvar>>,
    pub clingen: Option<crate::raw::Field<Clingen>>,
    pub chrom: Option<SourceChromosome>,
    pub hg19: Option<SourceSpan>,
    pub hg38: Option<SourceSpan>,
}
#[derive(Deserialize)]
pub(super) struct Dbsnp {
    pub rsid: Option<Field<OneOrMany<Field<biodata::Text>>>>,
    pub gene: Option<Field<OneOrMany<Field<Gene>>>>,
}
#[derive(Deserialize)]
pub(super) struct Gene {
    pub symbol: Option<Field<biodata::Text>>,
}
#[derive(Deserialize)]
pub(super) struct Clinvar {
    pub variant_id: Field<u64>,
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.total > 10_000 || self.hits.len() > 1000 || self.hits.len() as u64 > self.total {
            return Err(BadReply);
        }
        Ok(())
    }
}

impl Reply for Hit {
    fn check(&self) -> Result<(), BadReply> {
        if self
            .id
            .as_str()
            .parse::<biomcp_types::VariantInput>()
            .is_ok_and(|i| i.namespace() == biomcp_types::VariantNamespace::Chromosome)
        {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}

impl Hit {
    pub(super) fn matches(&self, query: &str) -> bool {
        if query.starts_with("rs") {
            self.dbsnp
                .as_ref()
                .and_then(Field::as_ref)
                .and_then(|d| d.rsid.as_ref())
                .and_then(Field::as_ref)
                .is_some_and(|ids| match ids {
                    OneOrMany::One(id) => id.as_ref().is_some_and(|id| id.as_str() == query),
                    OneOrMany::Many(ids) => ids
                        .iter()
                        .filter_map(Field::as_ref)
                        .any(|id| id.as_str() == query),
                })
        } else if let Some(id) = query.strip_prefix("clinvar.variant_id:") {
            self.clinvar
                .as_ref()
                .and_then(Field::as_ref)
                .and_then(|c| c.variant_id.as_ref())
                .is_some_and(|value| value.to_string() == id)
        } else {
            self.id.as_str() == query
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum SourceChromosome {
    Name(biodata::Text),
    Malformed(serde::de::IgnoredAny),
}
#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum SourceSpan {
    Bounds { start: u64, end: u64 },
    Malformed(serde::de::IgnoredAny),
}

#[derive(Debug, Deserialize)]
pub(super) struct Clingen {
    pub caid: crate::raw::Field<biodata::Text>,
}
