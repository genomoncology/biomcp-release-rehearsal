//! Keep association metadata failures separate from native row identity.
use super::raw::{EffectAllele, Trait};
use crate::native_text::NativeText;
use biodata::{FiniteNumber, Text};
use serde::Deserialize;
/// A source association retains publisher labels and numbers unchanged.
#[derive(Debug, Deserialize)]
pub(super) struct ReceivedAssociation {
    /// Stable association record identity.
    pub association_id: std::num::NonZeroU64,
    /// Catalog study accession.
    pub accession_id: Option<crate::raw::Field<Text>>,
    /// Source-reported traits.
    pub reported_trait: Option<crate::raw::Field<Vec<crate::raw::Field<NativeText>>>>,
    /// Source mapped ontology terms.
    pub efo_traits: Option<crate::raw::Field<Vec<crate::raw::Field<Trait>>>>,
    /// Source effect alleles, never interpreted as a classification.
    pub snp_allele: Option<crate::raw::Field<Vec<crate::raw::Field<EffectAllele>>>>,
    /// Reported association p value.
    pub p_value: Option<crate::raw::Field<FiniteNumber>>,
    /// Source exact mantissa, independent of rounded p_value.
    #[serde(rename = "pvalue_mantissa")]
    pub p_value_mantissa: Option<crate::raw::Field<u64>>,
    /// Source exponent, including values below the double range.
    #[serde(rename = "pvalue_exponent")]
    pub p_value_exponent: Option<crate::raw::Field<i64>>,
    /// Source author attribution.
    #[serde(rename = "first_author")]
    pub author: Option<crate::raw::Field<String>>,
    /// Source gene mapping strings.
    #[serde(rename = "mapped_genes")]
    pub mapped_gene: Option<crate::raw::Field<Vec<crate::raw::Field<NativeText>>>>,
    /// Native mapping locations.
    pub locations: Option<crate::raw::Field<Vec<crate::raw::Field<NativeText>>>>,
    /// Source interval lower bound.
    pub ci_lower: Option<crate::raw::Field<FiniteNumber>>,
    /// Source interval upper bound.
    pub ci_upper: Option<crate::raw::Field<FiniteNumber>>,
    /// Reported effect beta, preserving units and prose.
    pub beta: Option<crate::raw::Field<String>>,
    /// Reported odds ratio where provided.
    pub or_per_copy_num: Option<crate::raw::Field<FiniteNumber>>,
    /// Source allele frequency text, such as NR.
    pub risk_frequency: Option<crate::raw::Field<String>>,
    /// Source-reported effect interval.
    pub range: Option<crate::raw::Field<String>>,
    /// Source publication identity.
    pub pubmed_id: Option<crate::raw::Field<String>>,
}
impl From<ReceivedAssociation> for super::raw::Association {
    fn from(row: ReceivedAssociation) -> Self {
        let mut problems = crate::mapped::Problems {
            source: biomcp_types::Source::Gwas,
            record: crate::mapped::record(row.association_id),
            list: Vec::new(),
        };
        let accession_id = problems
            .field("accession_id", row.accession_id)
            .and_then(|v| {
                problems
                    .optional(
                        "accession_id",
                        Some(v.as_str().parse::<biodata::GwasStudyId>()),
                    )
                    .map(|_| v)
            });
        let reported_trait = problems
            .fields("reported_trait", row.reported_trait)
            .into_vec();
        let mut refusals = crate::Refusal::None;
        let efo_traits = problems
            .records("efo_traits", row.efo_traits, &mut refusals)
            .into_vec();
        let snp_allele = problems
            .records("snp_allele", row.snp_allele, &mut refusals)
            .into_vec();
        let mut value = Self {
            refusals,
            association_id: biodata::GwasAssociationId::from_nonzero(row.association_id),
            accession_id,
            reported_trait,
            efo_traits,
            snp_allele,
            p_value: problems.required_field("p_value", row.p_value),
            p_value_mantissa: problems.field("pvalue_mantissa", row.p_value_mantissa),
            p_value_exponent: problems.field("pvalue_exponent", row.p_value_exponent),
            author: problems.field("first_author", row.author),
            mapped_gene: row
                .mapped_gene
                .map(|v| problems.fields("mapped_genes", Some(v)).into_vec()),
            locations: row
                .locations
                .map(|v| problems.fields("locations", Some(v)).into_vec()),
            ci_lower: problems.field("ci_lower", row.ci_lower),
            ci_upper: problems.field("ci_upper", row.ci_upper),
            beta: problems.field("beta", row.beta),
            or_per_copy_num: problems.field("or_per_copy_num", row.or_per_copy_num),
            risk_frequency: problems.field("risk_frequency", row.risk_frequency),
            range: problems.field("range", row.range),
            pubmed_id: problems.field("pubmed_id", row.pubmed_id),
            problems: Vec::new(),
        };
        check_metadata(&mut value, &mut problems);
        value.problems = problems.list;
        value
    }
}

fn check_metadata(value: &mut super::raw::Association, problems: &mut crate::mapped::Problems) {
    if value.exact_p_value().is_none()
        && (value.p_value_mantissa.is_some() || value.p_value_exponent.is_some())
    {
        if !problems
            .list
            .iter()
            .any(|p| matches!(p.field.as_ref(), "pvalue_mantissa" | "pvalue_exponent"))
        {
            let field = if value.p_value_mantissa.is_none_or(|n| !(1..=9).contains(&n)) {
                "pvalue_mantissa"
            } else {
                "pvalue_exponent"
            };
            problems.required::<()>(field, None);
        }
        value.p_value_mantissa = None;
        value.p_value_exponent = None;
    }
    for trait_row in &mut value.efo_traits {
        for issue in trait_row.problems.drain(..) {
            crate::mapped::push_problem(
                &mut problems.list,
                problems.source,
                problems.record.clone(),
                &format!("efo_traits.{}", issue.field),
            );
        }
    }
    for allele in &mut value.snp_allele {
        for issue in allele.problems.drain(..) {
            crate::mapped::push_problem(
                &mut problems.list,
                problems.source,
                problems.record.clone(),
                &format!("snp_allele.{}", issue.field),
            );
        }
    }
    value.efo_traits.retain(|v| {
        let valid = v.efo_id.as_str().parse::<biodata::GwasTraitId>().is_ok();
        if !valid {
            value.refusals = crate::Refusal::Refused;
            problems.required::<()>("efo_traits.efo_id", None);
        }
        valid
    });
    value.snp_allele.retain(|v| {
        let valid = v.rs_id.as_str().parse::<biodata::RsId>().is_ok();
        if !valid {
            value.refusals = crate::Refusal::Refused;
            problems.required::<()>("snp_allele.rs_id", None);
        }
        valid
    });
}
