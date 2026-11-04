//! Native fields retain source spellings and independent error boundaries.
use super::annotation_raw::{PgxEvidenceTerm, PgxLocation, PgxObjectReference};
use crate::pgx_sources::pgx_row;
use biodata::ClinPgxAccession;
pgx_row! { PgxAllelePhenotype => biodata::PgxAllelePhenotype, record id {
    identity {
        id: i64 => "allelePhenotypes.id" as "ClinPGx",
    }
    values {
        allele: Box<str> => "allelePhenotypes.allele",
        #[serde(rename = "limitedEvidence")] limited_evidence: bool => "allelePhenotypes.limitedEvidence",
        phenotype: Box<str> => "allelePhenotypes.phenotype",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
    }
}}
pgx_row! { PgxAnnotationHistory => biodata::PgxAnnotationHistory, record id {
    identity {
        id: i64 => "history.id" as "ClinPGx",
    }
    values {
        date: Box<str> => "history.date",
        #[serde(rename = "type")] kind: Box<str> => "history.type",
        version: i64 => "history.version",
        description: Box<str> => "history.description",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
    }
}}
pgx_row! { PgxAnnotationDetail => biodata::PgxAnnotationDetail, record accession_id {
    identity {
        id: i64 => "id" as "ClinPGx",
        #[serde(rename = "accessionId")] accession_id: ClinPgxAccession => "accessionId",
    }
    values {
        #[serde(rename = "objCls")] obj_cls: Box<str> => "objCls",
        name: Box<str> => "name",
        #[serde(rename = "overrideLevel")] override_level: bool => "overrideLevel",
        pediatric: bool => "pediatric",
        #[serde(rename = "relatedChemicalsLogic")] related_chemicals_logic: Box<str> => "relatedChemicalsLogic",
        score: biodata::FiniteNumber => "score",
        #[serde(rename = "scoreDetails")] score_details: Box<str> => "scoreDetails",
        #[serde(rename = "overrideLevelDescription")] override_level_description: Box<str> => "overrideLevelDescription",
    }
    nested {
        #[serde(rename = "levelOfEvidence")] level_of_evidence: PgxEvidenceTerm => "levelOfEvidence",
        location: PgxLocation => "location",
    }
    lists {
        #[serde(rename = "conflictingVariantAnnotationIds")] conflicting_variant_annotation_ids: i64 => "conflictingVariantAnnotationIds" as "ClinPGx",
        types: Box<str> => "types",
    }
    nested_lists {
        #[serde(rename = "allelePhenotypes")] allele_phenotypes: PgxAllelePhenotype => "allelePhenotypes",
        history: PgxAnnotationHistory => "history",
        #[serde(rename = "relatedAlleles")] related_alleles: PgxObjectReference => "relatedAlleles",
        #[serde(rename = "relatedChemicals")] related_chemicals: PgxObjectReference => "relatedChemicals",
        #[serde(rename = "relatedDiseases")] related_diseases: PgxObjectReference => "relatedDiseases",
        #[serde(rename = "relatedGuidelines")] related_guidelines: PgxObjectReference => "relatedGuidelines",
        #[serde(rename = "relatedLabels")] related_labels: PgxObjectReference => "relatedLabels",
    }
    optional_lists {
    }
}}
