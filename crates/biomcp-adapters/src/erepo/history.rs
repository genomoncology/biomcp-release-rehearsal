//! Selected SEPIO documents supply their own facts; summaries supply routing only.
use crate::native_text::NativeText;
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::{Attribution, Text, VariantRef};
use biomcp_fetch::Read;
use biomcp_fetch::{BadReply, Freshness, Leg, PathSegment, Reply, RequestError};
use biomcp_types::{
    AssertionAssessment, AssertionCondition, AssertionDetail, AssertionDocument, AssertionDomain,
    AssertionKind, Source, VariantAssertion,
};
use serde::Deserialize;
const DOCUMENT: biomcp_fetch::Endpoint = super::SPEC
    .get("/evrepo/api/summary/classification")
    .fresh(Freshness::Days(1));
pub(super) fn fetch(
    leg: &Leg<'_>,
    caid: &str,
    record: &str,
    uuid: &str,
    version: &str,
) -> Result<Mapped<Option<VariantAssertion>>, RequestError> {
    let path = [uuid, "doc", "sepio", "version", version]
        .into_iter()
        .map(PathSegment::new)
        .collect::<Result<Vec<_>, _>>()
        .map_err(RequestError::Input)?;
    let endpoint = DOCUMENT.clone().with_segments(&path);
    let reply: DocumentReply = leg
        .call(
            &endpoint,
            biomcp_fetch::Input::Params(&[]),
            Read::json()
                .check(&|_| Ok(()))
                .cache_if(&|reply: &DocumentReply| {
                    reply
                        .data
                        .as_ref()
                        .is_some_and(|data| data.matches(caid, uuid, version))
                }),
        )?
        .value;
    let mut cx = Problems {
        source: Source::Erepo,
        record: Some(
            crate::mapped::text(record)
                .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Erepo, None))?,
        ),
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let value = match cx.field("document", Some(reply.data)) {
        Some(document) => match document.map(caid, record, uuid, version, &mut cx) {
            Some(row) => Some(row),
            None => {
                refusals = crate::Refusal::Refused;
                None
            }
        },
        None => {
            refusals = crate::Refusal::Refused;
            None
        }
    };
    Ok(Mapped {
        refusals,
        value,
        problems: cx.list,
    })
}
#[derive(Deserialize)]
struct DocumentReply {
    data: Field<Document>,
}
impl Reply for DocumentReply {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
struct Document {
    uuid: String,
    id: String,
    #[serde(rename = "type")]
    kind: String,
    metadata: Metadata,
    variant: Identifier,
    condition: Option<Field<Label>>,
    #[serde(rename = "statementOutcome")]
    outcome: Option<Field<Label>>,
    #[serde(rename = "evidenceLine")]
    lines: Option<Field<Vec<Field<Line>>>>,
}
#[derive(Deserialize)]
struct Metadata {
    version: String,
}
#[derive(Deserialize)]
struct Identifier {
    id: String,
}
#[derive(Deserialize)]
struct Label {
    label: Option<Field<NativeText>>,
    id: Option<Field<Text>>,
    #[serde(rename = "@id")]
    iri: Option<Field<Text>>,
}
#[derive(Deserialize)]
struct Line {
    #[serde(rename = "evidenceItem")]
    items: Option<Field<Vec<Field<Assessment>>>>,
}
#[derive(Deserialize)]
struct Assessment {
    #[serde(rename = "type")]
    kind: String,
    criterion: Option<Field<Criterion>>,
    #[serde(rename = "statementOutcome")]
    outcome: Option<Field<Label>>,
}
#[derive(Deserialize)]
struct Criterion {
    label: Option<Field<NativeText>>,
    id: Option<Field<Text>>,
    #[serde(rename = "@id")]
    iri: Option<Field<Text>>,
    #[serde(rename = "defaultStrength")]
    strength: Option<Field<Label>>,
}
impl Document {
    // Document identity is independent of its conclusion and evidence content.
    // https://dataexchange.clinicalgenome.org/interpretation/entities/VariantPathogenicityInterpretation.html
    fn matches(&self, caid: &str, uuid: &str, version: &str) -> bool {
        self.uuid == uuid
            && self.metadata.version == version
            && self.variant.id == format!("CAR:{caid}")
            && self.kind == "VariantPathogenicityInterpretation"
            && !self.id.trim().is_empty()
    }
    fn map(
        self,
        caid: &str,
        record: &str,
        uuid: &str,
        version: &str,
        cx: &mut Problems,
    ) -> Option<VariantAssertion> {
        if !self.matches(caid, uuid, version) {
            cx.required::<()>("document", None);
            return None;
        }
        let attribution = Attribution {
            approved: None,
            published: None,
            source: Source::Erepo.text(), record: cx.text("PCERDocID", Some(record)), version: cx.text("docVersion", Some(version)),
            release: None, submitter: None, evaluated: None,
            url: cx.text("uuid", Some(&format!("https://erepo.clinicalgenome.org/evrepo/ui/classification/{}?version={version}", self.uuid))),
        };
        let conditions = cx
            .field("condition", self.condition)
            .map(|condition| AssertionCondition {
                database: None,
                accession: None,
                id: cx
                    .field("condition.id", condition.id)
                    .and_then(|id| cx.optional("condition.id", Some(id.as_str().parse()))),
                label: cx.source_field("condition.label", condition.label),
            })
            .into_iter()
            .collect();
        let mut assessments = Vec::new();
        for line in cx.fields("evidenceLine", self.lines).into_vec() {
            for item in cx.fields("evidenceItem", line.items).into_vec() {
                if item.kind == "CriterionAssessment" {
                    assessments.push(item.map(cx));
                }
            }
        }
        let outcome = cx.field("statementOutcome", self.outcome);
        let (label, outcome_id, outcome_iri) = match outcome {
            Some(outcome) => (
                cx.source_field("statementOutcome.label", outcome.label),
                cx.field("statementOutcome.id", outcome.id),
                cx.field("statementOutcome.@id", outcome.iri),
            ),
            None => (None, None, None),
        };
        let detail = AssertionDetail {
            conditions,
            document: Some(AssertionDocument {
                uuid: cx.text("uuid", Some(&self.uuid))?,
                native_id: cx.text("id", Some(&self.id))?,
                document_type: cx.text("type", Some(&self.kind))?,
                outcome_id,
                outcome_iri,
                assessments: assessments.into(),
            }),
            ..Default::default()
        };
        Some(match label {
            Some(label) => crate::assertion::row(
                AssertionKind::Erepo,
                AssertionDomain::Germline,
                label,
                attribution,
                VariantRef::allele(cx.optional("caId", Some(caid.parse()))?),
                detail,
                None,
            ),
            None => crate::assertion::missing(
                AssertionKind::Erepo,
                AssertionDomain::Germline,
                attribution,
                detail,
            ),
        })
    }
}
impl Assessment {
    fn map(self, cx: &mut Problems) -> AssertionAssessment {
        let criterion = cx.field("criterion", self.criterion);
        let (criterion, criterion_id, criterion_iri, strength) = match criterion {
            Some(criterion) => (
                cx.source_field("criterion.label", criterion.label),
                cx.field("criterion.id", criterion.id),
                cx.field("criterion.@id", criterion.iri),
                cx.field("criterion.defaultStrength", criterion.strength),
            ),
            None => (None, None, None, None),
        };
        let (default_strength, default_strength_id, default_strength_iri) = match strength {
            Some(strength) => (
                cx.source_field("defaultStrength.label", strength.label),
                cx.field("defaultStrength.id", strength.id),
                cx.field("defaultStrength.@id", strength.iri),
            ),
            None => (None, None, None),
        };
        let outcome = cx.field("statementOutcome", self.outcome);
        let (outcome, outcome_id, outcome_iri) = match outcome {
            Some(outcome) => (
                cx.source_field("statementOutcome.label", outcome.label),
                cx.field("statementOutcome.id", outcome.id),
                cx.field("statementOutcome.@id", outcome.iri),
            ),
            None => (None, None, None),
        };
        // Preserve contrary SEPIO label/identifier pairs without repairing them.
        // https://dataexchange.clinicalgenome.org/interpretation/entities/CriterionAssessment.html
        let outcome_conflict = [outcome_id.as_ref(), outcome_iri.as_ref()]
            .into_iter()
            .flatten()
            .any(|id| {
                matches!(
                    (id.as_str(), outcome.as_ref().map(Text::as_str)),
                    ("SEPIO:0000224", Some("Met")) | ("SEPIO:0000223", Some("Not Met"))
                )
            });
        AssertionAssessment {
            criterion,
            criterion_id,
            criterion_iri,
            default_strength,
            default_strength_id,
            default_strength_iri,
            outcome,
            outcome_id,
            outcome_iri,
            outcome_conflict,
        }
    }
}
