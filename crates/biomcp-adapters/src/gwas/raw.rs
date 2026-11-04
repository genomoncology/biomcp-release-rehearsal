use crate::native_text::NativeText;
use biodata::{FiniteNumber, Text};

use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Response {
    #[serde(rename = "_embedded")]
    pub embedded: Option<Embedded>,
    #[serde(rename = "_links")]
    pub(super) links: Links,
    pub page: Page,
}
#[derive(Debug, Deserialize)]
pub(super) struct Embedded {
    pub associations: Vec<super::scope::Row>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Links {
    #[serde(rename = "self")]
    pub(super) current: Link,
}
#[derive(Debug, Deserialize)]
pub(super) struct Link {
    pub(super) href: Text,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Page {
    pub(super) size: u32,
    pub total_elements: u64,
    total_pages: u64,
    pub(super) number: u32,
}
/// A source association retains publisher labels and numbers unchanged.
#[derive(Debug, Deserialize)]
#[serde(from = "super::receive::ReceivedAssociation")]
pub(super) struct Association {
    /// Actual refused native child records.
    pub refusals: crate::Refusal,
    /// Rejected source metadata attributed to the association.
    pub problems: Vec<biomcp_types::Problem>,
    /// Stable association record identity.
    pub association_id: biodata::GwasAssociationId,
    /// Catalog study accession.
    pub accession_id: Option<Text>,
    /// Source-reported traits.
    pub reported_trait: Vec<NativeText>,
    /// Source mapped ontology terms.
    pub efo_traits: Vec<Trait>,
    /// Source effect alleles, never interpreted as a classification.
    pub snp_allele: Vec<EffectAllele>,
    /// Reported association p value.
    pub p_value: Option<FiniteNumber>,
    /// Source exact mantissa, independent of rounded p_value.
    #[serde(rename = "pvalue_mantissa")]
    pub p_value_mantissa: Option<u64>,
    /// Source exponent, including values below the double range.
    #[serde(rename = "pvalue_exponent")]
    pub p_value_exponent: Option<i64>,
    /// Source author attribution.
    #[serde(rename = "first_author")]
    pub author: Option<String>,
    /// Source gene mapping strings.
    #[serde(rename = "mapped_genes")]
    pub mapped_gene: Option<Vec<NativeText>>,
    /// Native mapping locations.
    pub locations: Option<Vec<NativeText>>,
    /// Source interval lower bound.
    pub ci_lower: Option<FiniteNumber>,
    /// Source interval upper bound.
    pub ci_upper: Option<FiniteNumber>,
    /// Reported effect beta, preserving units and prose.
    pub beta: Option<String>,
    /// Reported odds ratio where provided.
    pub or_per_copy_num: Option<FiniteNumber>,
    /// Source allele frequency text, such as NR.
    pub risk_frequency: Option<String>,
    /// Source-reported effect interval.
    pub range: Option<String>,
    /// Source publication identity.
    pub pubmed_id: Option<String>,
}
/// Source trait identity and label.
#[derive(Debug, Deserialize)]
#[serde(from = "ReceivedTrait")]
pub(super) struct Trait {
    /// Rejected display metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Source ontology ID.
    pub efo_id: Text,
    /// Whole source label.
    pub efo_trait: Option<biodata::Text>,
}
#[derive(Debug, Deserialize)]
struct ReceivedTrait {
    /// Source ontology ID.
    pub efo_id: Text,
    /// Whole source label.
    pub efo_trait: Option<crate::raw::Field<NativeText>>,
}
impl From<ReceivedTrait> for Trait {
    fn from(row: ReceivedTrait) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Gwas,
            record: None,
            list: Vec::new(),
        };
        let value = problems.required_field("efo_trait", row.efo_trait);
        Self {
            efo_id: row.efo_id,
            efo_trait: value.map(Into::into),
            problems: problems.list,
        }
    }
}

/// Effect allele qualified by its exact source rsID.
#[derive(Debug, Deserialize)]
#[serde(from = "ReceivedEffectAllele")]
pub(super) struct EffectAllele {
    /// Rejected display metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Source rsID.
    pub rs_id: Text,
    /// Source effect allele text, not normalized.
    pub effect_allele: Option<Text>,
}
#[derive(Debug, Deserialize)]
struct ReceivedEffectAllele {
    /// Source rsID.
    pub rs_id: Text,
    /// Source effect allele text, not normalized.
    pub effect_allele: Option<crate::raw::Field<Text>>,
}
impl From<ReceivedEffectAllele> for EffectAllele {
    fn from(row: ReceivedEffectAllele) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Gwas,
            record: None,
            list: Vec::new(),
        };
        let value = problems.required_field("effect_allele", row.effect_allele);
        Self {
            rs_id: row.rs_id,
            effect_allele: value,
            problems: problems.list,
        }
    }
}

impl Reply for Response {
    fn check(&self) -> Result<(), BadReply> {
        let rows = self
            .embedded
            .as_ref()
            .map(|e| e.associations.as_slice())
            .unwrap_or_default();
        if self.page.size == 0
            || rows.len() > self.page.size as usize
            || self.page.total_pages != self.page.total_elements.div_ceil(self.page.size.into())
            || self.page.total_elements == 0 && !rows.is_empty()
            || self.page.total_elements > 0 && self.embedded.is_none()
        {
            return Err(BadReply);
        }
        let offset = u64::from(self.page.number) * u64::from(self.page.size);
        let expected = self
            .page
            .total_elements
            .saturating_sub(offset)
            .min(self.page.size.into());
        if rows.len() as u64 != expected {
            return Err(BadReply);
        }
        let mut ids = std::collections::BTreeSet::new();
        if rows
            .iter()
            .filter_map(|row| row.value.as_ref())
            .any(|r| !ids.insert(r.association_id))
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
impl Response {
    pub(super) fn identities_admitted(&self) -> bool {
        self.embedded.as_ref().is_none_or(|embedded| {
            embedded
                .associations
                .iter()
                .all(|row| row.value.as_ref().is_some())
        })
    }
    pub(super) fn matches(&self, id: &str, page: u32, size: u32, url: &str) -> bool {
        self.page.number == page
            && self.page.size == size
            && self.links.current.href.as_str() == url
            && self.embedded.as_ref().is_none_or(|e| {
                e.associations
                    .iter()
                    .all(|a| a.selectors.alleles().any(|s| s.rs_id.as_str() == id))
            })
    }
}

impl Association {
    /// Decode exact native parts without converting the probability to a double.
    pub(super) fn exact_p_value(&self) -> Option<biodata::ScientificPValue> {
        let mantissa = self.p_value_mantissa?;
        format!("{mantissa}e{}", self.p_value_exponent?)
            .parse()
            .ok()
    }
}
