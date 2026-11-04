//! Native fields retain source spellings and independent error boundaries.
// Native IDs identify records; malformed nested content does not refuse its parent.
// https://api.clinpgx.org/openapi.json
use crate::pgx_sources::pgx_row;
use biodata::{ClinPgxAccession, PgxUnobserved};
pgx_row! { PgxObjectReference => biodata::PgxObjectReference, record id {
    identity {
        id: ClinPgxAccession => "references.id",
    }
    values {
        #[serde(rename = "objCls")] obj_cls: Box<str> => "references.objCls",
        name: Box<str> => "references.name",
        symbol: Box<str> => "references.symbol",
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
pgx_row! { PgxCrossReference => biodata::PgxCrossReference, record id {
    identity {
        id: i64 => "crossReferences.id" as "ClinPGx",
    }
    values {
        resource: Box<str> => "crossReferences.resource",
        #[serde(rename = "resourceId")] resource_id: Box<str> => "crossReferences.resourceId",
        _url: Box<str> => "crossReferences._url",
        name: Box<str> => "crossReferences.name",
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
pgx_row! { PgxDiplotypeReference => biodata::PgxDiplotypeReference, record id {
    identity {
        id: i64 => "diplotypes.id" as "ClinPGx",
    }
    values {
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
pgx_row! { PgxLocation => biodata::PgxLocation, record id {
    identity {
        id: i64 => "location.id" as "ClinPGx",
    }
    values {
        #[serde(rename = "buildVersion")] build_version: Box<str> => "location.buildVersion",
        #[serde(rename = "chromosomeId")] chromosome_id: Box<str> => "location.chromosomeId",
        #[serde(rename = "chromosomeName")] chromosome_name: Box<str> => "location.chromosomeName",
        #[serde(rename = "copyNumber")] copy_number: Box<str> => "location.copyNumber",
        #[serde(rename = "displayName")] display_name: Box<str> => "location.displayName",
        #[serde(rename = "gpPosition")] gp_position: i64 => "location.gpPosition",
        #[serde(rename = "refSeqPosition")] ref_seq_position: i64 => "location.refSeqPosition",
        rsid: Box<str> => "location.rsid",
        #[serde(rename = "tagGene")] tag_gene: bool => "location.tagGene",
        #[serde(rename = "type")] kind: Box<str> => "location.type",
    }
    nested {
        #[serde(rename = "refSeqCrossReference")] ref_seq_cross_reference: PgxCrossReference => "location.refSeqCrossReference",
        variant: PgxObjectReference => "location.variant",
    }
    lists {
    }
    nested_lists {
        diplotypes: PgxDiplotypeReference => "location.diplotypes",
        genes: PgxObjectReference => "location.genes",
        haplotypes: PgxObjectReference => "location.haplotypes",
        #[serde(rename = "linkedObjects")] linked_objects: PgxObjectReference => "location.linkedObjects",
    }
    optional_lists {
    }
}}
pgx_row! { PgxEvidenceTerm => biodata::PgxEvidenceTerm, record id {
    identity {
        id: i64 => "levelOfEvidence.id" as "ClinPGx",
    }
    values {
        resource: Box<str> => "levelOfEvidence.resource",
        term: Box<str> => "levelOfEvidence.term",
        #[serde(rename = "termId")] term_id: Box<str> => "levelOfEvidence.termId",
        valid: bool => "levelOfEvidence.valid",
    }
    nested {
    }
    lists {
    }
    nested_lists {
    }
    optional_lists {
        synonyms: PgxUnobserved => "levelOfEvidence.synonyms",
    }
}}
pgx_row! { PgxAnnotationHeader => biodata::PgxAnnotationHeader, record accession_id {
    identity {
        id: i64 => "id" as "ClinPGx",
        #[serde(rename = "accessionId")] accession_id: ClinPgxAccession => "accessionId",
    }
    values {
        name: Box<str> => "name",
        #[serde(rename = "relatedChemicalsLogic")] related_chemicals_logic: Box<str> => "relatedChemicalsLogic",
    }
    nested {
        #[serde(rename = "levelOfEvidence")] level_of_evidence: PgxEvidenceTerm => "levelOfEvidence",
        location: PgxLocation => "location",
    }
    lists {
        types: Box<str> => "types",
    }
    nested_lists {
        #[serde(rename = "relatedChemicals")] related_chemicals: PgxObjectReference => "relatedChemicals",
        #[serde(rename = "relatedGuidelines")] related_guidelines: PgxObjectReference => "relatedGuidelines",
        #[serde(rename = "relatedLabels")] related_labels: PgxObjectReference => "relatedLabels",
    }
    optional_lists {
    }
}}
pgx_row! { PgxAnnotationReference => biodata::PgxAnnotationReference, record id {
    identity {
        #[serde(rename = "objCls")] obj_cls: Box<str> => "objCls",
        id: ClinPgxAccession => "id",
    }
    values {
        name: Box<str> => "name",
    }
    nested {
    }
    lists {
        #[serde(rename = "crossReferences")] cross_references: PgxUnobserved => "crossReferences",
    }
    nested_lists {
    }
    optional_lists {
    }
}}
