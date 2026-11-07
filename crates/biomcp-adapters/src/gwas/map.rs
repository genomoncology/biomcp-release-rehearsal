//! One mapping shared by association search and the existing variant section.
use biomcp_types::{GwasAssociation, GwasEffectAllele};
pub(super) fn association(
    row: super::raw::Association,
    problems: &mut Vec<biomcp_types::Problem>,
) -> GwasAssociation {
    let exact_p_value = row.exact_p_value();
    problems.extend(row.problems);
    let record = row.association_id.get().to_string().parse().ok();
    GwasAssociation {
        id: row.association_id,
        study: row.accession_id,
        traits: row.reported_trait.into_iter().map(Into::into).collect(),
        ontology_traits: row
            .efo_traits
            .into_iter()
            .map(|t| biomcp_types::GwasTrait::new(t.efo_id, t.efo_trait))
            .collect(),
        alleles: row
            .snp_allele
            .into_iter()
            .map(|a| GwasEffectAllele {
                rs_id: a.rs_id,
                allele: a.effect_allele,
            })
            .collect(),
        // A value below the float range reads 0.0; the exact value is then the only truth.
        p_value: row
            .p_value
            .filter(|p| p.get() != 0.0 || exact_p_value.is_none()),
        exact_p_value,
        author: optional_text(row.author.clone(), "first_author", record.clone(), problems),
        mapped_genes: row
            .mapped_gene
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect(),
        locations: row
            .locations
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect(),
        ci_lower: row.ci_lower,
        ci_upper: row.ci_upper,
        beta: optional_text(row.beta, "beta", record.clone(), problems),
        odds_ratio: row.or_per_copy_num,
        risk_frequency: optional_text(
            row.risk_frequency,
            "risk_frequency",
            record.clone(),
            problems,
        ),
        interval: optional_text(row.range, "range", record.clone(), problems),
        publication: optional_text(row.pubmed_id, "pubmed_id", record.clone(), problems),
    }
}

fn optional_text(
    value: Option<String>,
    field: &str,
    record: Option<biodata::Text>,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Option<biodata::Text> {
    let value = value?;
    let parsed = if field == "pubmed_id" {
        value
            .parse::<biodata::Pmid>()
            .map_err(|_| biodata::Error::ArticleIdInvalid)
            .and_then(|_| biodata::Text::new(&value))
    } else {
        biodata::Text::from_source(&value)
    };
    match parsed {
        Ok(value) => Some(value),
        Err(_) => {
            let problem = biomcp_types::Problem {
                source: biomcp_types::Source::Gwas,
                record,
                field: field.into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            };
            if !problems.contains(&problem) {
                problems.push(problem);
            }
            None
        }
    }
}

/// Accepted association values and source diagnostics after native admission.
#[derive(Debug, serde::Deserialize)]
#[serde(from = "super::raw::Association")]
pub struct Association {
    /// Mapped association identity and facts.
    pub value: GwasAssociation,
    /// Rejected native optional values.
    pub problems: Vec<biomcp_types::Problem>,
    /// Actual refused native child identities.
    pub refusals: crate::Refusal,
}
impl From<super::raw::Association> for Association {
    fn from(row: super::raw::Association) -> Self {
        let refusals = row.refusals;
        let mut problems = Vec::new();
        let value = association(row, &mut problems);
        Self {
            value,
            problems,
            refusals,
        }
    }
}
