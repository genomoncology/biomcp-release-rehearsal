//! Separate the service envelope from source candidate admission and optional values.
use crate::{mapped::Problems, raw::Field};
use biodata::{SourceText, Text};
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub(super) struct Response {
    pub flag: Text,
    pub metadata: BTreeMap<String, Field<SourceText>>,
    #[serde(flatten, deserialize_with = "ordered_variants")]
    pub variants: Vec<(String, Field<RawVariant>)>,
}
pub(super) type Loci = BTreeMap<String, Field<Locus>>;
#[derive(Deserialize)]
pub(super) struct RawVariant {
    pub validation_warnings: Option<Field<Vec<Field<SourceText>>>>,
    pub hgvs_lrg_transcript_variant: Option<Field<SourceText>>,
    pub hgvs_lrg_variant: Option<Field<SourceText>>,
    pub hgvs_refseqgene_variant: Option<Field<SourceText>>,
    pub hgvs_predicted_protein_consequence: Option<Field<BTreeMap<String, Field<SourceText>>>>,
    pub alt_genomic_loci: Option<Field<Vec<Field<Loci>>>>,
    pub submitted_variant: String,
    pub hgvs_transcript_variant: String,
    pub primary_assembly_loci: Option<Field<Loci>>,
}
#[derive(Deserialize)]
pub(super) struct Locus {
    pub hgvs_genomic_description: Field<Text>,
}
impl RawVariant {
    fn admitted(&self, input: &str, warning: bool) -> bool {
        self.submitted_variant == input
            && (warning
                || Text::new(&self.hgvs_transcript_variant)
                    .is_ok_and(|value| biodata::Expression::hgvs(value).is_ok()))
    }
}
impl Response {
    pub(super) fn eligible(&self, input: &str) -> bool {
        self.variants
            .iter()
            .all(|(_, row)| row.as_ref().is_some_and(|row| row.admitted(input, false)))
    }
    pub(super) fn records(
        self,
        input: &str,
        problems: &mut Problems,
        refusals: &mut crate::Refusal,
    ) -> Vec<(String, RawVariant)> {
        let warning = self.flag.as_str() == "warning";
        self.variants
            .into_iter()
            .filter_map(|(key, raw)| {
                problems.record = crate::mapped::record(&key);
                let Some(row) = problems.field("identity", Some(raw)) else {
                    *refusals = crate::Refusal::Refused;
                    return None;
                };
                if row.admitted(input, warning) {
                    Some((key, row))
                } else {
                    *refusals = crate::Refusal::Refused;
                    problems.required::<()>("identity", None);
                    None
                }
            })
            .collect()
    }
}
impl Reply for Response {
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if (200..300).contains(&status)
            || matches!(status, 400 | 422) && self.flag.as_str() == "warning"
        {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
    fn check(&self) -> Result<(), BadReply> {
        if self.variants.is_empty()
            || self.variants.len() > 1000
            || !self.metadata.contains_key("variantvalidator_version")
        {
            return Err(BadReply);
        }
        Ok(())
    }
}

fn ordered_variants<'de, D: serde::Deserializer<'de>>(
    de: D,
) -> Result<Vec<(String, Field<RawVariant>)>, D::Error> {
    struct Ordered;
    impl<'de> serde::de::Visitor<'de> for Ordered {
        type Value = Vec<(String, Field<RawVariant>)>;
        fn expecting(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            out.write_str("source candidate map")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut rows = Vec::new();
            while let Some(row) = map.next_entry()? {
                rows.push(row);
            }
            Ok(rows)
        }
    }
    de.deserialize_map(Ordered)
}
