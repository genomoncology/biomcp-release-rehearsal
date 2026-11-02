use super::{variant_map::identity, variant_raw as raw};
use crate::{mapped::Problems, raw::Field};
use biodata::{GeneSymbol, HgvsText, Text, VariantRef};
use biomcp_types::{CivicMember, SourceRecord};
pub(super) fn member(raw: raw::Member, cx: &mut Problems) -> Option<(CivicMember, Option<Text>)> {
    let id = identity(raw.id, "variants.id", cx)?;
    cx.record = crate::mapped::record(id);
    let before = cx.list.len();
    let label = cx.source_field("variants.name", raw.name);
    let feature = cx.field("feature", raw.feature);
    let (feature, feature_record) = if let Some(feature) = feature {
        let name = cx.source_field("feature.name", feature.name);
        let id = cx.field("feature.id", feature.id).and_then(|id| {
            cx.optional(
                "feature.id",
                Some(if raw::id(id) {
                    format!("civic-feature:{id}").parse()
                } else {
                    Err(biodata::Error::CurieInvalid)
                }),
            )
        });
        (name, id)
    } else {
        (None, None)
    };
    let hgvs_descriptions = cx.fields("hgvsDescriptions", raw.hgvs_descriptions);
    let caid = cx.field("alleleRegistryId", raw.allele_registry_id);
    let complete = cx.list.len() == before;
    let subject = if complete {
        member_subject(label.as_ref(), feature.as_ref(), &hgvs_descriptions, cx)
    } else {
        None
    };
    let link = cx.field("variants.link", raw.link);
    Some((
        CivicMember {
            record: record(id, link.as_ref(), cx)?,
            label,
            feature,
            feature_record,
            hgvs_descriptions,
            subject,
        },
        caid,
    ))
}
fn member_subject(
    label: Option<&Text>,
    feature: Option<&Text>,
    forms: &[Text],
    cx: &mut Problems,
) -> Option<VariantRef> {
    let label = label?;
    // Source-only features without protein notation retain their native representation.
    let proteins: Vec<_> = forms
        .iter()
        .filter(|form| {
            form.as_str()
                .split_once(':')
                .is_some_and(|(_, body)| body.starts_with("p."))
        })
        .collect();
    if proteins.is_empty() {
        return None;
    }
    let proteins = proteins
        .into_iter()
        .map(|form| cx.optional("hgvsDescriptions", Some(form.as_str().parse::<HgvsText>())))
        .collect::<Option<Vec<_>>>()?;
    let symbol = cx.optional(
        "feature.name",
        Some(feature?.as_str().parse::<GeneSymbol>()),
    )?;
    let gene = cx.optional(
        "feature.name",
        Some(biodata::SourceGeneRef::try_from(symbol)),
    )?;
    match proteins.as_slice() {
        [] => None,
        [protein] => Some(VariantRef::protein_hgvs(
            gene,
            protein.clone(),
            Some(label.clone()),
        )),
        _ => Some(VariantRef::protein(gene, label.clone())),
    }
}
pub(super) fn subject(
    record: &SourceRecord,
    label: Option<&Text>,
    raw_name: Option<&Text>,
    compound: Option<bool>,
    members: Vec<&CivicMember>,
    complete: bool,
    cx: &mut Problems,
) -> Option<VariantRef> {
    if !complete {
        return None;
    }
    match (compound, members.as_slice()) {
        (Some(false), [member]) => member.subject.clone(),
        (Some(true), [_, ..]) => {
            let label = label?;
            let native = members
                .iter()
                .map(|member| {
                    let id = cx.optional(
                        "variants.id",
                        Some(format!("civic:{}", member.record.id).parse()),
                    )?;
                    cx.optional(
                        "variants.subject",
                        Some(VariantRef::source_with_feature(
                            id,
                            member.feature.clone()?,
                            member.feature_record.clone(),
                            member.subject.clone().into_iter().collect(),
                        )),
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            let expression = cx.required_field("rawName", raw_name.cloned().map(Field::Value))?;
            let expression = cx.optional(
                "rawName",
                Some(biodata::ProfileExpression::civic(expression)),
            )?;
            cx.optional(
                "molecular_profile.expression",
                Some(VariantRef::profile_expression(
                    record.id.clone(),
                    label.clone(),
                    native.into(),
                    expression,
                )),
            )
        }
        _ => None,
    }
}
pub(super) fn record(id: u64, link: Option<&Text>, cx: &mut Problems) -> Option<SourceRecord> {
    let id = cx.optional("id", Some(Text::new(&id.to_string())))?;
    let url = cx.optional(
        "link",
        link.map(|link| {
            if !link.as_str().starts_with('/') {
                return Err(());
            }
            Text::new(&format!("https://civicdb.org{link}")).map_err(|_| ())
        }),
    );
    Some(SourceRecord {
        namespace: biodata::IdentifierNamespace::Unknown,
        id,
        url,
    })
}
