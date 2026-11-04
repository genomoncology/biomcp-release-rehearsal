//! Native claim identity survives malformed optional clinical content.
use super::{
    claim_content::{citations, condition, evidence_level, native, term, therapies},
    claim_vocab as vocab, variant_map, variant_raw as raw,
};
use crate::mapped::Problems;
use biodata::{Direction, Term, Text, VariantRef};
#[path = "claim_statement.rs"]
mod statement;
use biomcp_types::{
    CivicKind, CivicNativeClaim, CivicSignificance, CivicStatement, CivicStatus, CivicTherapy,
    DrugHeader, SourceRecord,
};

pub(super) struct MappedClaim {
    pub claim: CivicStatement,
    pub therapies: Box<[DrugHeader]>,
}
struct Claim {
    id: u64,
    kind: Option<CivicKind>,
    status: Option<CivicStatus>,
    significance: Option<CivicSignificance>,
    direction: Option<Text>,
    native: CivicNativeClaim,
    therapies: Box<[DrugHeader]>,
    assertion: bool,
    context_bad: bool,
    amp_level: Option<Text>,
    interaction: Option<Text>,
}
pub(super) fn evidence(
    raw: raw::Evidence,
    subject: Option<&VariantRef>,
    context_bad: bool,
    cx: &mut Problems,
) -> Option<MappedClaim> {
    let id = variant_map::identity(raw.id, "id", cx)?;
    cx.record = Text::new(&format!("EID{id}")).into_iter().next();
    let before = cx.list.len();
    let kind = cx
        .field("evidenceType", raw.evidence_type)
        .map(|v| CivicKind::from(v.as_str().to_owned()));
    let status = cx.field("status", raw.status);
    let significance = cx
        .field("significance", raw.significance)
        .map(|v| CivicSignificance::from_evidence(v.as_str().to_owned()));
    let direction = cx.field("evidenceDirection", raw.evidence_direction);
    let condition = condition(raw.disease, cx);
    let therapies = therapies(raw.therapies, cx);
    let interaction = interaction(raw.therapy_interaction_type, &therapies.0, cx);
    let context_bad = context_bad || before != cx.list.len();
    let strength = cx
        .field("evidenceLevel", raw.evidence_level)
        .map(|label| evidence_level(label, cx));
    let citations = citations(raw.source, cx);
    let native = native(
        significance.as_ref().and_then(|v| term(v.as_str(), cx)),
        strength,
        condition,
        &therapies.0,
        None,
        citations,
    );
    build(
        Claim {
            id,
            kind,
            status,
            significance,
            direction,
            native,
            therapies: therapies.1,
            assertion: false,
            context_bad,
            amp_level: None,
            interaction,
        },
        subject,
        cx,
    )
}
pub(super) fn assertion(
    raw: raw::Assertion,
    subject: Option<&VariantRef>,
    context_bad: bool,
    cx: &mut Problems,
) -> Option<MappedClaim> {
    let id = variant_map::identity(raw.id, "id", cx)?;
    cx.record = Text::new(&format!("AID{id}")).into_iter().next();
    let before = cx.list.len();
    let kind = cx
        .field("assertionType", raw.assertion_type)
        .map(|v| CivicKind::from_assertion(v.as_str().to_owned()));
    let status = cx.field("status", raw.status);
    let significance = cx
        .field("significance", raw.significance)
        .map(|v| CivicSignificance::from_assertion(v.as_str().to_owned()));
    let direction = cx.field("assertionDirection", raw.assertion_direction);
    let condition = condition(raw.disease, cx);
    let therapies = therapies(raw.therapies, cx);
    let interaction = interaction(raw.therapy_interaction_type, &therapies.0, cx);
    let origin = cx
        .field("variantOrigin", raw.variant_origin)
        .map(|label| Term {
            label,
            codings: Box::default(),
        });
    let amp = cx.field("ampLevel", raw.amp_level);
    let (classification, strength) =
        if let Some(label) = amp.as_ref().filter(|label| label.as_str() != "NA") {
            match vocab::amp(Some(label)) {
                Some(value) => (Some(value.0), value.1),
                None => {
                    cx.required::<()>("ampLevel", None);
                    (
                        Some(Term {
                            label: label.clone(),
                            codings: Box::default(),
                        }),
                        None,
                    )
                }
            }
        } else {
            (
                significance.as_ref().and_then(|v| term(v.as_str(), cx)),
                None,
            )
        };
    let native = native(
        classification,
        strength,
        condition,
        &therapies.0,
        origin,
        Box::default(),
    );
    build(
        Claim {
            id,
            kind,
            status,
            significance,
            direction,
            native,
            therapies: therapies.1,
            assertion: true,
            context_bad: context_bad || before != cx.list.len(),
            amp_level: amp,
            interaction,
        },
        subject,
        cx,
    )
}
fn interaction(
    raw: Option<crate::raw::Field<Text>>,
    therapies: &[CivicTherapy],
    cx: &mut Problems,
) -> Option<Text> {
    let value = cx.field("therapyInteractionType", raw);
    if therapies.len() > 1 {
        if value
            .as_ref()
            .is_none_or(|v| !matches!(v.as_str(), "COMBINATION" | "SEQUENTIAL" | "SUBSTITUTES"))
        {
            cx.required::<()>("therapyInteractionType", None);
        }
    } else if value.is_some() {
        cx.required::<()>("therapyInteractionType", None);
    }
    value
}
fn build(mut row: Claim, subject: Option<&VariantRef>, cx: &mut Problems) -> Option<MappedClaim> {
    let id = cx.optional(
        "id",
        Some(Text::new(&format!(
            "{}{}",
            if row.assertion { "AID" } else { "EID" },
            row.id
        ))),
    )?;
    let url = cx.optional(
        "id",
        Some(Text::new(&format!(
            "https://civicdb.org/{}/{}",
            if row.assertion {
                "assertions"
            } else {
                "evidence"
            },
            row.id
        ))),
    )?;
    cx.record = Some(id.clone());
    row.native.attribution.record = Some(id.clone());
    row.native.attribution.url = Some(url.clone());
    let direction = vocabulary(&mut row, cx);
    let statement = statement::common(&row, subject, direction);
    if statement.is_none() && !row.context_bad && row.native.therapies.len() < 2 {
        cx.required::<()>("statement", None);
    }
    Some(MappedClaim {
        claim: CivicStatement {
            statement_unmapped: statement.is_none(),
            native: statement.is_none().then_some(row.native),
            statement,
            record: SourceRecord {
                namespace: biodata::IdentifierNamespace::Unknown,
                id,
                url: Some(url),
            },
            kind: row.kind,
            status: row.status,
            direction: row.direction,
            significance: row.significance,
            amp_level: row.amp_level,
            therapy_interaction_type: row.interaction,
        },
        therapies: row.therapies,
    })
}

fn vocabulary(row: &mut Claim, cx: &mut Problems) -> Option<Direction> {
    for (unknown, field) in [
        (
            row.kind.as_ref().is_some_and(CivicKind::is_unknown),
            if row.assertion {
                "assertionType"
            } else {
                "evidenceType"
            },
        ),
        (
            row.status.as_ref().is_some_and(CivicStatus::is_unknown),
            "status",
        ),
        (
            row.significance
                .as_ref()
                .is_some_and(CivicSignificance::is_unknown),
            "significance",
        ),
    ] {
        if unknown {
            cx.required::<()>(field, None);
            row.context_bad = true;
        }
    }
    match row.direction.as_ref().map(Text::as_str) {
        Some("SUPPORTS") => Some(Direction::Supports),
        Some("DOES_NOT_SUPPORT") => Some(Direction::Disputes),
        Some(_) => {
            cx.required::<()>(
                if row.assertion {
                    "assertionDirection"
                } else {
                    "evidenceDirection"
                },
                None,
            );
            None
        }
        None => None,
    }
}
