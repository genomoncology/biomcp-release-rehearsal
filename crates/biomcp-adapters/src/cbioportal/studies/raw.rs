use biodata::{
    CbioportalCancerTypeId, Pmid, Study, StudyHeader, StudyId, StudyImportTime, StudyResourceCount,
    Text,
};
use biomcp_fetch::{BadReply, KeyKind, Keys, Row, encode_bundle_row};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
/// Checked native catalog row, retaining card fields for later exact reads.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", from = "ReceivedCatalogRow")]
pub(super) struct CatalogRow {
    #[serde(with = "crate::mapped::refusal_serde")]
    refusals: crate::Refusal,
    problems: Vec<biomcp_types::Problem>,
    study_id: StudyId,
    name: Option<biodata::Text>,
    cancer_type_id: Option<CbioportalCancerTypeId>,
    description: Option<Box<str>>,
    all_sample_count: Option<u64>,
    reference_genome: Option<Text>,
    pmid: Option<Box<str>>,
    citation: Option<Box<str>>,
    sequenced_sample_count: Option<u64>,
    cna_sample_count: Option<u64>,
    structural_variant_count: Option<u64>,
    treatment_count: Option<u64>,
    resource_counts: Box<[StudyResourceCount]>,
    import_date: Option<StudyImportTime>,
    groups: Box<str>,
    read_permission: bool,
    public_study: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReceivedCatalogRow {
    #[serde(
        default = "crate::mapped::refusal_serde::none",
        with = "crate::mapped::refusal_serde"
    )]
    refusals: crate::Refusal,
    problems: Option<Vec<biomcp_types::Problem>>,
    study_id: StudyId,
    name: Option<crate::raw::Field<crate::native_text::NativeText>>,
    cancer_type_id: Option<crate::raw::Field<CbioportalCancerTypeId>>,
    description: Option<crate::raw::Field<Box<str>>>,
    all_sample_count: Option<crate::raw::Field<u64>>,
    reference_genome: Option<crate::raw::Field<Text>>,
    pmid: Option<crate::raw::Field<Box<str>>>,
    citation: Option<crate::raw::Field<Box<str>>>,
    sequenced_sample_count: Option<crate::raw::Field<u64>>,
    cna_sample_count: Option<crate::raw::Field<u64>>,
    structural_variant_count: Option<crate::raw::Field<u64>>,
    treatment_count: Option<crate::raw::Field<u64>>,
    resource_counts: Vec<super::resource::Resource>,
    import_date: Option<crate::raw::Field<StudyImportTime>>,
    groups: Box<str>,
    read_permission: bool,
    public_study: bool,
}
impl From<ReceivedCatalogRow> for CatalogRow {
    fn from(row: ReceivedCatalogRow) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Cbioportal,
            record: crate::mapped::record(&row.study_id),
            list: Vec::new(),
        };
        // Source admission clears this field; only stored catalog rows restore diagnostics.
        if let Some(stored) = row.problems {
            problems.list = stored;
        }
        let mut refusals = row.refusals;
        Self {
            study_id: row.study_id,
            name: problems.required_field("name", row.name).map(Into::into),
            cancer_type_id: problems.required_field("cancerTypeId", row.cancer_type_id),
            description: problems.required_field("description", row.description),
            all_sample_count: problems.required_field("allSampleCount", row.all_sample_count),
            reference_genome: problems.required_field("referenceGenome", row.reference_genome),
            pmid: problems.field("pmid", row.pmid),
            citation: problems.field("citation", row.citation),
            sequenced_sample_count: problems
                .required_field("sequencedSampleCount", row.sequenced_sample_count),
            cna_sample_count: problems.required_field("cnaSampleCount", row.cna_sample_count),
            structural_variant_count: problems
                .required_field("structuralVariantCount", row.structural_variant_count),
            treatment_count: problems.required_field("treatmentCount", row.treatment_count),
            resource_counts: row
                .resource_counts
                .into_iter()
                .filter_map(|r| r.mapped(&mut problems, &mut refusals))
                .collect(),
            import_date: problems.required_field("importDate", row.import_date),
            groups: row.groups,
            read_permission: row.read_permission,
            public_study: row.public_study,
            problems: problems.list,
            refusals,
        }
    }
}
impl CatalogRow {
    pub(super) fn header(&self) -> Result<crate::Mapped<StudyHeader>, BadReply> {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Cbioportal,
            record: crate::mapped::record(&self.study_id),
            list: self.problems.clone(),
        };
        let value = StudyHeader {
            study_id: self.study_id.clone(),
            name: self.name.clone(),
            cancer_type_id: self.cancer_type_id.clone(),
            all_sample_count: self.all_sample_count,
            pmids: self.pmid.as_deref().map(|s| {
                s.split(',')
                    .filter_map(|v| problems.optional("pmid", Some(v.trim().parse::<Pmid>())))
                    .collect()
            }),
            citation: self.citation.clone(),
            reference_genome: self.reference_genome.clone(),
        };
        Ok(crate::Mapped {
            refusals: self.refusals,
            value,
            problems: problems.list,
        })
    }
    pub(super) fn matches(&self, q: &biomcp_types::StudySearch) -> bool {
        q.cancer_type()
            .is_none_or(|v| self.cancer_type_id.as_ref() == Some(v))
            && q.text().is_none_or(|v| {
                let needle = v.as_str().to_lowercase();
                [
                    Some(self.study_id.as_str()),
                    self.name.as_ref().map(|v| v.as_str()),
                    self.description.as_deref(),
                ]
                .into_iter()
                .flatten()
                .any(|v| v.to_lowercase().contains(&needle))
            })
    }
    pub(super) fn card(self) -> Result<crate::Mapped<Study>, BadReply> {
        let header = self.header()?;
        let value = Study {
            header: header.value,
            description: self.description,
            sequenced_sample_count: self.sequenced_sample_count,
            cna_sample_count: self.cna_sample_count,
            structural_variant_count: self.structural_variant_count,
            treatment_count: self.treatment_count,
            patient_count: None,
            resource_counts: self.resource_counts,
            import_date: self.import_date,
            groups: self.groups,
            read_permission: self.read_permission,
            public_study: self.public_study,
        };
        Ok(crate::Mapped {
            refusals: header.refusals,
            value,
            problems: header.problems,
        })
    }
}
impl Row for CatalogRow {
    const REQUIRE_ACCEPTED_ROWS: bool = true;
    const MAX_BYTES: u64 = 1024 * 1024;
    fn decode_document(body: &[u8]) -> Result<Vec<String>, BadReply> {
        if body.len() > 1024 * 1024 {
            return Err(BadReply);
        }
        let source_rows: Vec<Box<biomcp_fetch::RawValue>> =
            biomcp_fetch::decode_bundle_document(body)?;
        let mut ids = BTreeSet::new();
        let mut result = Vec::new();
        for source_row in source_rows {
            let context: super::context::Context =
                biomcp_fetch::decode_bundle_document(source_row.get().as_bytes())?;
            context.check()?;
            let received: Result<ReceivedCatalogRow, _> =
                biomcp_fetch::decode_bundle_document(source_row.get().as_bytes());
            let mut received = match received {
                Ok(row) => row,
                Err(_) => {
                    // Preserve malformed identity bytes for the shared decoder and its source position.
                    result.push(source_row.get().to_owned());
                    continue;
                }
            };
            received.problems = None;
            received.refusals = crate::Refusal::None;
            let row = Self::from(received);
            if !ids.insert(row.study_id.clone()) {
                return Err(BadReply);
            }
            row.header()?;
            result.push(encode_bundle_row(&row)?);
        }
        Ok(result)
    }
    fn keys(&self) -> Keys {
        Keys {
            exact: vec![
                (KeyKind::Id, self.study_id.to_string().into()),
                (KeyKind::Band, "catalog".into()),
            ],
            words: vec![],
        }
    }
}
