use super::{drug_raw as raw, variant_claims, variant_map};
use crate::{mapped::Problems, raw::Field};
use biodata::VariantRef;
use biomcp_types::{DrugCivicClaim, DrugCivicMember, DrugCivicProfile, DrugGet};

pub(super) fn context(
    raw: Option<Field<raw::Profile>>,
    cx: &mut Problems,
) -> Option<(DrugCivicProfile, Option<VariantRef>)> {
    let raw = cx.field("molecularProfile", raw)?;
    let id = variant_map::identity(raw.id, "molecularProfile.id", cx)?;
    let link = cx.field("molecularProfile.link", raw.link);
    let record = variant_map::record(id, link.as_ref(), cx)?;
    cx.record = Some(record.id.clone());
    let before = cx.list.len();
    let label = cx.source_field("molecularProfile.name", raw.name);
    let raw_name = cx.field("rawName", raw.raw_name);
    let compound = cx.field("isComplex", raw.is_complex);
    let count = raw.variants.as_ref().and_then(Field::as_ref).map(Vec::len);
    let mut members = Vec::new();
    let parent = cx.record.clone();
    for row in cx.fields("variants", raw.variants).into_vec() {
        cx.record = parent.clone();
        let reference_build = cx
            .field("coordinates", row.coordinates)
            .and_then(|c| cx.field("referenceBuild", c.reference_build));
        if let Some((member, _)) = variant_map::member(row.variant, cx) {
            members.push(DrugCivicMember {
                member,
                reference_build,
            });
        }
    }
    cx.record = parent;
    let complete = before == cx.list.len() && count == Some(members.len());
    let subject = variant_map::subject(
        &record,
        label.as_ref(),
        raw_name.as_ref(),
        compound,
        members.iter().map(|m| &m.member).collect(),
        complete,
        cx,
    );
    Some((
        DrugCivicProfile {
            record,
            label,
            raw_name,
            subject: subject.clone(),
            compound,
            members: members.into(),
        },
        subject,
    ))
}
fn build(
    context: Option<(DrugCivicProfile, Option<VariantRef>)>,
    mapped: variant_claims::MappedClaim,
    interaction: Option<biodata::Text>,
    query: &DrugGet,
    cx: &mut Problems,
) -> Option<DrugCivicClaim> {
    if mapped.claim.status.as_ref().is_some_and(|status| {
        !status.is_unknown() && !matches!(status, biomcp_types::CivicStatus::Accepted)
    }) {
        cx.required::<()>("status", None);
        return None;
    }
    let requested = query.input().to_string();
    if !mapped
        .therapies
        .iter()
        .any(|t| t.id.to_string() == requested)
    {
        cx.required::<()>("therapies.id", None);
        return None;
    }
    if query.civic_profile().is_some_and(|id| {
        context
            .as_ref()
            .is_none_or(|(p, _)| p.record.id.as_str() != id.to_string())
    }) {
        cx.required::<()>("molecularProfile.id", None);
        return None;
    }
    Some(DrugCivicClaim {
        profile: context.map(|v| v.0),
        claim: mapped.claim,
        therapies: mapped.therapies,
        therapy_interaction_type: interaction,
        oncokb_missing: Box::default(),
        oncokb_lookup: None,
    })
}
pub(super) fn evidence(
    raw: raw::Evidence,
    query: &DrugGet,
    cx: &mut Problems,
) -> Option<DrugCivicClaim> {
    let before = cx.list.len();
    let context = context(raw.molecular_profile, cx);
    let context_bad = before != cx.list.len();
    let mapped = variant_claims::evidence(
        raw.claim,
        context.as_ref().and_then(|v| v.1.as_ref()),
        context_bad,
        cx,
    )?;
    let interaction = mapped.claim.therapy_interaction_type.clone();
    build(context, mapped, interaction, query, cx)
}
pub(super) fn assertion(
    raw: raw::Assertion,
    query: &DrugGet,
    cx: &mut Problems,
) -> Option<DrugCivicClaim> {
    let before = cx.list.len();
    let context = context(raw.molecular_profile, cx);
    let context_bad = before != cx.list.len();
    let mapped = variant_claims::assertion(
        raw.claim,
        context.as_ref().and_then(|v| v.1.as_ref()),
        context_bad,
        cx,
    )?;
    let interaction = mapped.claim.therapy_interaction_type.clone();
    build(context, mapped, interaction, query, cx)
}
