use crate::mapped::Problems;
use crate::native_text::NativeText;
use crate::raw::Field;
use biodata::Text;
use biomcp_fetch::{BadReply, Reply};
use biomcp_types::{GwasAncestry, GwasCountry, GwasLocus, GwasReportedGene};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Links {
    #[serde(rename = "self")]
    pub current: Link,
}
#[derive(Debug, Deserialize)]
pub(super) struct Link {
    pub href: Text,
}
pub(super) fn owned(url: &Text, prefix: &str) -> bool {
    url.as_str()
        .strip_prefix(prefix)
        .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
}
#[derive(Debug, Deserialize)]
pub(super) struct Ancestries {
    #[serde(rename = "_embedded")]
    pub embedded: AncestryRows,
}
#[derive(Debug, Deserialize)]
pub(super) struct AncestryRows {
    pub ancestries: Vec<Ancestry>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Ancestry {
    #[serde(rename = "type")]
    stage: Option<Field<Text>>,
    number_of_individuals: Option<Field<u64>>,
    ancestral_groups: Option<Field<Vec<Field<Group>>>>,
    country_of_origin: Option<Field<Vec<Field<Country>>>>,
    country_of_recruitment: Option<Field<Vec<Field<Country>>>>,
    #[serde(rename = "_links")]
    pub links: Links,
}
#[derive(Debug, Deserialize)]
struct Group {
    ancestral_group: Option<Field<NativeText>>,
}
#[derive(Debug, Deserialize)]
struct Country {
    country_name: Option<Field<NativeText>>,
    major_area: Option<Field<NativeText>>,
    region: Option<Field<NativeText>>,
}
impl Country {
    fn row(self, field: &str, problems: &mut Problems) -> GwasCountry {
        GwasCountry {
            country: problems
                .required_field(&format!("{field}.country_name"), self.country_name)
                .map(Into::into),
            major_area: problems
                .field(&format!("{field}.major_area"), self.major_area)
                .map(Into::into),
            region: problems
                .field(&format!("{field}.region"), self.region)
                .map(Into::into),
        }
    }
}
impl Ancestry {
    pub(super) fn row(
        self,
        problems: &mut Problems,
        refusals: &mut crate::Refusal,
    ) -> GwasAncestry {
        problems.record = Some(self.links.current.href.clone());
        GwasAncestry {
            stage: problems.required_field("type", self.stage),
            number_of_individuals: problems
                .field("number_of_individuals", self.number_of_individuals),
            ancestry_groups: problems
                .records("ancestral_groups", self.ancestral_groups, refusals)
                .into_vec()
                .into_iter()
                .filter_map(|g| {
                    match problems
                        .required_field("ancestral_groups.ancestral_group", g.ancestral_group)
                    {
                        Some(value) => Some(value.into()),
                        None => {
                            *refusals = crate::Refusal::Refused;
                            None
                        }
                    }
                })
                .collect(),
            country_of_origin: problems
                .records("country_of_origin", self.country_of_origin, refusals)
                .into_vec()
                .into_iter()
                .map(|r| r.row("country_of_origin", problems))
                .collect(),
            country_of_recruitment: problems
                .records(
                    "country_of_recruitment",
                    self.country_of_recruitment,
                    refusals,
                )
                .into_vec()
                .into_iter()
                .map(|r| r.row("country_of_recruitment", problems))
                .collect(),
            source_url: self.links.current.href,
        }
    }
}
impl Reply for Ancestries {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Loci {
    #[serde(rename = "_embedded")]
    pub embedded: LocusRows,
}
#[derive(Debug, Deserialize)]
pub(super) struct LocusRows {
    pub loci: Vec<Locus>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Locus {
    description: Option<Field<NativeText>>,
    strongest_risk_alleles: Option<Field<Vec<Field<Allele>>>>,
    author_reported_genes: Option<Field<Vec<Field<Gene>>>>,
    #[serde(rename = "_links")]
    pub links: Links,
}
#[derive(Debug, Deserialize)]
struct Allele {
    risk_allele_name: Option<Field<Text>>,
}
#[derive(Debug, Deserialize)]
struct Gene {
    gene_name: Option<Field<NativeText>>,
    ensembl_gene_ids: Option<Field<Vec<Field<Text>>>>,
    entrez_gene_ids: Option<Field<Vec<Field<Text>>>>,
}
impl Locus {
    pub(super) fn row(self, problems: &mut Problems, refusals: &mut crate::Refusal) -> GwasLocus {
        problems.record = Some(self.links.current.href.clone());
        GwasLocus {
            description: problems
                .field("description", self.description)
                .map(Into::into),
            strongest_risk_alleles: problems
                .records(
                    "strongest_risk_alleles",
                    self.strongest_risk_alleles,
                    refusals,
                )
                .into_vec()
                .into_iter()
                .filter_map(|a| {
                    match problems.required_field(
                        "strongest_risk_alleles.risk_allele_name",
                        a.risk_allele_name,
                    ) {
                        Some(value) => Some(value),
                        None => {
                            *refusals = crate::Refusal::Refused;
                            None
                        }
                    }
                })
                .collect(),
            author_reported_genes: problems
                .records(
                    "author_reported_genes",
                    self.author_reported_genes,
                    refusals,
                )
                .into_vec()
                .into_iter()
                .map(|g| GwasReportedGene {
                    name: problems
                        .field("author_reported_genes.gene_name", g.gene_name)
                        .map(Into::into),
                    ensembl_ids: problems
                        .fields("author_reported_genes.ensembl_gene_ids", g.ensembl_gene_ids),
                    entrez_ids: problems
                        .fields("author_reported_genes.entrez_gene_ids", g.entrez_gene_ids),
                })
                .collect(),
            source_url: Some(self.links.current.href),
        }
    }
}
impl Reply for Loci {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
