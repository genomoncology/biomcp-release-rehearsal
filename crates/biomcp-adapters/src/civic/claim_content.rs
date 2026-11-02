use super::variant_raw as raw;
use crate::mapped::Problems;
use biodata::{Attribution, Term, Text};
use biomcp_types::{
    CivicCondition, CivicNativeClaim, CivicTherapy, DrugHeader, DrugSource, Source,
};
pub(super) fn term(value: &str, cx: &mut Problems) -> Option<Term> {
    cx.text("significance", Some(value)).map(|label| Term {
        label,
        codings: Box::default(),
    })
}
pub(super) fn condition(
    raw: Option<crate::raw::Field<raw::Disease>>,
    cx: &mut Problems,
) -> Option<CivicCondition> {
    let raw = cx.field("disease", raw)?;
    let label = cx.source_field("disease.name", raw.name);
    let id = cx
        .field("disease.doid", raw.doid)
        .and_then(|id| cx.optional("disease.doid", Some(format!("DOID:{id}").parse())));
    Some(CivicCondition { id, label })
}
pub(super) fn therapies(
    raw: Option<crate::raw::Field<Vec<crate::raw::Field<raw::Therapy>>>>,
    cx: &mut Problems,
) -> (Box<[CivicTherapy]>, Box<[DrugHeader]>) {
    let mut native = Vec::new();
    let mut headers = Vec::new();
    for row in cx.fields("therapies", raw).into_vec() {
        let id = cx.field("therapies.id", row.id).and_then(|id| {
            cx.optional(
                "therapies.id",
                Some(if raw::id(id) { Ok(id) } else { Err(()) }),
            )
        });
        let label = cx.source_field("therapies.name", row.name);
        if let Some(id) = id
            && let Some(id) = cx.optional("therapies.id", Some(id.to_string().parse()))
        {
            headers.push(DrugHeader {
                id,
                source: DrugSource::Civic,
                name: label.clone(),
                identity_level: biodata::DrugIdentityLevel::Unknown,
                matched_name: None,
                r#type: None,
                version: None,
                url: None,
            });
        }
        native.push(CivicTherapy {
            id: id.map(|id| biodata::Identifier {
                namespace: cx.text("therapies.id.namespace", Some("CIViC")).into(),
                id,
            }),
            label,
        });
    }
    (native.into(), headers.into())
}
pub(super) fn citations(
    raw: Option<crate::raw::Field<raw::Citation>>,
    cx: &mut Problems,
) -> Box<[biodata::Pmid]> {
    let Some(raw) = cx.field("source", raw) else {
        return Box::default();
    };
    let kind = cx.field("source.sourceType", raw.source_type);
    let id = cx.field("source.citationId", raw.citation_id);
    if kind.as_ref().is_some_and(|kind| kind.as_str() == "PUBMED") {
        cx.optional("source.citationId", id.map(|id| id.as_str().parse()))
            .into_iter()
            .collect()
    } else {
        Box::default()
    }
}
pub(super) fn evidence_level(label: Text, cx: &mut Problems) -> Term {
    use biodata::CivicEvidenceLevelCode as L;
    let code = match label.as_str() {
        "A" => Some(L::A),
        "B" => Some(L::B),
        "C" => Some(L::C),
        "D" => Some(L::D),
        "E" => Some(L::E),
        _ => {
            cx.required::<()>("evidenceLevel", None);
            None
        }
    };
    Term {
        label,
        codings: code
            .map(biodata::Coding::CivicEvidenceLevel)
            .into_iter()
            .collect(),
    }
}
pub(super) fn native(
    classification: Option<Term>,
    strength: Option<Term>,
    condition: Option<CivicCondition>,
    therapies: &[CivicTherapy],
    allele_origin: Option<Term>,
    citations: Box<[biodata::Pmid]>,
) -> CivicNativeClaim {
    CivicNativeClaim {
        classification,
        strength,
        condition,
        therapies: therapies.into(),
        allele_origin,
        citations,
        attribution: Attribution {
            approved: None,
            published: None,
            source: Source::Civic.text(),
            record: None,
            version: None,
            release: None,
            submitter: None,
            evaluated: None,
            url: None,
        },
    }
}
