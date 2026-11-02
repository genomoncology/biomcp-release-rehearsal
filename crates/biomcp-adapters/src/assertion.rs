use biodata::{Attribution, Coding, Proposition, Statement, Term, Text, VariantRef};

use biomcp_types::{
    AssertionDetail, AssertionDomain, AssertionKind, UnmappedAssertion, VariantAssertion,
};
/// Map only the reviewed source categories, retaining every other source label natively.
pub(crate) fn row(
    kind: AssertionKind,
    domain: AssertionDomain,
    label: Text,
    attribution: Attribution,
    subject: VariantRef,
    detail: AssertionDetail,
    review: Option<Text>,
) -> VariantAssertion {
    let proposition = proposition(domain, &label);
    let mut row = VariantAssertion {
        kind,
        domain,
        statement_unmapped: proposition.is_none(),
        statement: None,
        unmapped: None,
        detail: Some(detail),
    };
    if let Some(proposition) = proposition {
        let coding = acmg(&label, row.detail.as_ref());
        let clinvar = attribution.source.as_str() == "clinvar";
        let mut statement = Statement::new(
            subject.into(),
            proposition,
            Term {
                label,
                codings: coding.into_iter().collect(),
            },
            attribution,
        );
        statement.strength = review.map(|label| {
            let coding = clinvar
                .then(|| biodata::ReviewStatus::from_source_label(label.as_str()))
                .flatten()
                .map(Coding::ClinvarReviewStatus);
            Term {
                label,
                codings: coding.into_iter().collect(),
            }
        });
        if let Some(detail) = &row.detail {
            if detail.conditions.len() == 1 {
                statement.condition = detail.conditions.first().and_then(|c| c.concept());
            }
            if detail.origins.len() == 1 {
                statement.allele_origin = detail.origins.first().cloned().map(|label| {
                    let coding = clinvar
                        .then(|| biodata::AlleleOrigin::from_source_label(label.as_str()))
                        .flatten()
                        .map(Coding::ClinvarAlleleOrigin);
                    Term {
                        label,
                        codings: coding.into_iter().collect(),
                    }
                });
            }
            statement.citations = detail.citations.clone();
        }
        row.statement = Some(statement);
    } else {
        row.unmapped = Some(UnmappedAssertion {
            classification: Some(label),
            attribution,
            review_status: review,
            conditions: row
                .detail
                .as_ref()
                .map(|d| d.conditions.clone())
                .unwrap_or_default(),
        });
    }
    row
}
/// Retain native document provenance without fabricating a missing conclusion.
pub(crate) fn missing(
    kind: AssertionKind,
    domain: AssertionDomain,
    attribution: Attribution,
    detail: AssertionDetail,
) -> VariantAssertion {
    VariantAssertion {
        kind,
        domain,
        statement_unmapped: true,
        statement: None,
        unmapped: Some(UnmappedAssertion {
            classification: None,
            attribution,
            review_status: None,
            conditions: detail.conditions.clone(),
        }),
        detail: Some(detail),
    }
}
// Native category mapping follows NCBI's germline vocabulary and ClinGen GKM G.01.
// https://www.ncbi.nlm.nih.gov/clinvar/docs/clinsig/
// https://dataexchange.clinicalgenome.org/clinvar-gkm/profiles/classifications/
// Whole-label ASCII casing tolerance applies only to proposition selection.
fn proposition(domain: AssertionDomain, label: &Text) -> Option<Proposition> {
    match domain {
        AssertionDomain::SomaticClinicalImpact => Some(Proposition::ClinicalSignificance),
        AssertionDomain::Oncogenicity => Some(Proposition::Oncogenicity),
        AssertionDomain::Germline => match label.as_str().to_ascii_lowercase().as_str() {
            "pathogenic"
            | "likely pathogenic"
            | "uncertain significance"
            | "likely benign"
            | "benign"
            | "pathogenic/likely pathogenic"
            | "benign/likely benign"
            | "conflicting classifications of pathogenicity"
            | "pathogenic, low penetrance"
            | "likely pathogenic, low penetrance"
            | "established risk allele"
            | "likely risk allele"
            | "uncertain risk allele" => Some(Proposition::Pathogenicity),
            _ => None,
        },
    }
}
fn acmg(label: &Text, detail: Option<&AssertionDetail>) -> Option<Coding> {
    let detail = detail?;
    if !detail
        .methods
        .iter()
        .any(|m| m.as_str() == "ACMG Guidelines, 2015")
        || !detail.citations.iter().any(|p| p.to_string() == "25741868")
    {
        return None;
    }
    use biodata::AcmgAmp2015Code as C;
    let code = match label.as_str() {
        "Pathogenic" => C::Pathogenic,
        "Likely pathogenic" => C::LikelyPathogenic,
        "Uncertain significance" => C::UncertainSignificance,
        "Likely benign" => C::LikelyBenign,
        "Benign" => C::Benign,
        _ => return None,
    };
    Some(Coding::AcmgAmp2015(code))
}
