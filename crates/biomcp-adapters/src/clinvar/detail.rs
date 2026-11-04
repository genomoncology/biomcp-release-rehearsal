use crate::xml_element::Element;
use biodata::Text;
use biomcp_fetch::BadReply;
use biomcp_types::{AssertionCondition, AssertionConditionRelationship, AssertionDetail};
pub(super) fn rcv_conditions(
    record: Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<Vec<AssertionCondition>, BadReply> {
    let Some(list) = record.child("ClassifiedConditionList")? else {
        return Ok(Vec::new());
    };
    list.children("ClassifiedCondition")?
        .into_iter()
        .map(|c| condition(c.attr("DB")?, c.attr("ID")?, Some(c.text()?), problems))
        .collect()
}
fn condition(
    db: Option<String>,
    id: Option<String>,
    label: Option<String>,
    problems: &mut crate::mapped::Problems,
) -> Result<AssertionCondition, BadReply> {
    let qualified = problems.optional(
        "conditions.id",
        match (&db, &id) {
            (_, Some(id)) if id.contains(':') => Some(id.parse()),
            (Some(db), Some(id)) => Some(format!("{db}:{id}").parse()),
            _ => None,
        },
    );
    Ok(AssertionCondition {
        database: problems.checked_text("conditions.database", db.as_deref()),
        accession: problems.checked_text("conditions.accession", id.as_deref()),
        id: qualified,
        label: problems.text("conditions.label", label.as_deref()),
    })
}
pub(super) fn submission(
    record: Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<AssertionDetail, BadReply> {
    let mut detail = AssertionDetail::default();
    let mut methods = Vec::new();
    let mut citations = Vec::new();
    for set in record.children("AttributeSet")? {
        for attribute in set.children("Attribute")? {
            if attribute.attr("Type")?.as_deref() == Some("AssertionMethod") {
                methods.extend(problems.text("methods", Some(&attribute.text()?)));
            }
        }
        for citation in set.children("Citation")? {
            for id in citation.children("ID")? {
                if id.attr("Source")?.as_deref() == Some("PubMed") {
                    citations.extend(
                        problems.optional("citations", Some(id.text()?.parse::<biodata::Pmid>())),
                    );
                }
            }
        }
    }
    detail.methods = methods.into();
    detail.citations = citations.into();
    detail.conditions = conditions(record, problems)?.into();
    detail.condition_relationships = relationships(record, problems)?.into();
    let mut origins = Vec::<Text>::new();
    if let Some(observations) = record.child("ObservedInList")? {
        for observation in observations.children("ObservedIn")? {
            if let Some(value) = observation.value(&["Sample", "Origin"])? {
                let Some(value) = problems.text("origins", Some(&value)) else {
                    continue;
                };
                if !origins.contains(&value) {
                    origins.push(value);
                }
            }
        }
    }
    detail.origins = origins.into();
    Ok(detail)
}
pub(super) fn conditions(
    record: Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<Vec<AssertionCondition>, BadReply> {
    let mut conditions = Vec::new();
    for set in record.children("TraitSet")? {
        for trait_ in set.children("Trait")? {
            conditions.extend(trait_conditions(trait_, problems)?);
        }
    }
    Ok(conditions)
}
fn relationships(
    record: Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<Vec<AssertionConditionRelationship>, BadReply> {
    let mut relationships = Vec::new();
    for set in record.children("TraitSet")? {
        for trait_ in set.children("Trait")? {
            for related in trait_.children("TraitRelationship")? {
                relationships.push(AssertionConditionRelationship {
                    relationship_type: problems.text(
                        "condition_relationships.relationship_type",
                        related.attr("Type")?.as_deref(),
                    ),
                    source_conditions: trait_conditions(trait_, problems)?.into(),
                    related_conditions: trait_conditions(related, problems)?.into(),
                });
            }
        }
    }
    Ok(relationships)
}
fn trait_conditions(
    trait_: Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<Vec<AssertionCondition>, BadReply> {
    let mut preferred = None;
    let mut labels = Vec::new();
    for name in trait_.children("Name")? {
        if let Some(value) = name.child("ElementValue")? {
            let label = value.text()?;
            if value.attr("Type")?.as_deref() == Some("Preferred") {
                preferred = Some(label.clone());
            }
            labels.push(label);
        }
    }
    let label = preferred.or_else(|| {
        if labels.len() == 1 {
            labels.pop()
        } else {
            None
        }
    });
    let refs = trait_.children("XRef")?;
    if refs.is_empty() {
        return if label.is_some() {
            Ok(vec![condition(None, None, label, problems)?])
        } else {
            Ok(Vec::new())
        };
    }
    refs.into_iter()
        .map(|reference| {
            condition(
                reference.attr("DB")?,
                reference.attr("ID")?,
                label.clone(),
                problems,
            )
        })
        .collect()
}
