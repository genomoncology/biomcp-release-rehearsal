use biodata::{Predicate, Proposition, Term, Text};
pub(super) fn proposition(kind: &str) -> Option<Proposition> {
    // https://docs.civicdb.org/en/latest/model/evidence/type.html
    match kind {
        "PREDICTIVE" => Some(Proposition::TherapeuticResponse),
        "DIAGNOSTIC" => Some(Proposition::Diagnostic),
        "PROGNOSTIC" => Some(Proposition::Prognostic),
        "PREDISPOSING" => Some(Proposition::Pathogenicity),
        "ONCOGENIC" => Some(Proposition::Oncogenicity),
        "FUNCTIONAL" => Some(Proposition::FunctionalImpact),
        _ => None,
    }
}
pub(super) fn predicate(kind: &str, significance: &str) -> Option<Predicate> {
    // https://docs.civicdb.org/en/latest/model/evidence/significance.html
    match (kind, significance) {
        ("PREDICTIVE", "SENSITIVITYRESPONSE") => Some(Predicate::PredictsSensitivityTo),
        ("PREDICTIVE", "RESISTANCE") => Some(Predicate::PredictsResistanceTo),
        ("DIAGNOSTIC", "POSITIVE") => Some(Predicate::IsDiagnosticInclusionCriterionFor),
        ("DIAGNOSTIC", "NEGATIVE") => Some(Predicate::IsDiagnosticExclusionCriterionFor),
        ("PROGNOSTIC", "BETTER_OUTCOME") => Some(Predicate::AssociatedWithBetterOutcomeFor),
        ("PROGNOSTIC", "POOR_OUTCOME") => Some(Predicate::AssociatedWithWorseOutcomeFor),
        ("FUNCTIONAL", value) if value != "NA" => Some(Predicate::ImpactsFunctionOf),
        _ => None,
    }
}
pub(super) fn amp(label: Option<&Text>) -> Option<(Term, Option<Term>)> {
    use biodata::{AmpAscoCap2017LevelCode as L, AmpAscoCap2017TierCode as T, Coding};

    let label = label?;
    let (tier, level) = match label.as_str() {
        "TIER_I_LEVEL_A" => (T::TierI, Some(L::LevelA)),
        "TIER_I_LEVEL_B" => (T::TierI, Some(L::LevelB)),
        "TIER_II_LEVEL_C" => (T::TierIi, Some(L::LevelC)),
        "TIER_II_LEVEL_D" => (T::TierIi, Some(L::LevelD)),
        "TIER_III" => (T::TierIii, None),
        "TIER_IV" => (T::TierIv, None),
        _ => return None,
    };
    Some((
        Term {
            label: label.clone(),
            codings: Box::new([Coding::AmpAscoCap2017Tier(tier)]),
        },
        level.map(|level| Term {
            label: label.clone(),
            codings: Box::new([Coding::AmpAscoCap2017Level(level)]),
        }),
    ))
}
