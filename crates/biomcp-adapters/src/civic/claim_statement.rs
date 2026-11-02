use super::{Claim, vocab};
use biodata::{Concept, Direction, Statement, VariantRef};
pub(super) fn common(
    row: &Claim,
    subject: Option<&VariantRef>,
    direction: Option<Direction>,
) -> Option<Statement> {
    (!row.context_bad && row.native.therapies.len() < 2)
        .then_some(subject)
        .flatten()
        .zip(
            row.kind
                .as_ref()
                .and_then(|kind| vocab::proposition(kind.as_str())),
        )
        .zip(row.native.classification.clone())
        .filter(|_| direction.is_some() && row.significance.is_some())
        .map(|((subject, proposition), classification)| Statement {
            method: None,
            mode_of_inheritance: None,
            criteria_not_met: Box::default(),
            status: None,
            subject: subject.clone().into(),
            proposition,
            predicate: row.kind.as_ref().zip(row.significance.as_ref()).and_then(
                |(kind, significance)| vocab::predicate(kind.as_str(), significance.as_str()),
            ),
            direction,
            classification,
            strength: row.native.strength.clone(),
            condition: row.native.condition.as_ref().and_then(|c| {
                c.label.clone().map(|label| Concept {
                    id: c.id.clone(),
                    label,
                })
            }),
            therapies: row
                .native
                .therapies
                .iter()
                .filter_map(|t| t.label.clone().map(|label| Concept { id: None, label }))
                .collect(),
            allele_origin: row.native.allele_origin.clone(),
            gene_context: None,
            criteria: Box::default(),
            citations: row.native.citations.clone(),
            attribution: row.native.attribution.clone(),
        })
}
