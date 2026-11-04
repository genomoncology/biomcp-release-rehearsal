use super::raw::{Annotation, Document};
use crate::mapped::push_problem;
use crate::mapped::record as record_text;
use crate::{Mapped, raw::Field};
use biodata::{Pmid, Text};

use biomcp_types::{
    AnnotatedEntity, AnnotationKind, Annotations, Capped, Problem, Section, Source,
};
const LIMIT: usize = 20;
pub(super) fn document(id: Pmid, document: Option<&Document>) -> Mapped<Section<Annotations>> {
    let mut problems = Vec::new();
    let mut entities: Vec<(String, AnnotatedEntity)> = Vec::new();
    let mut refusals = crate::Refusal::None;
    for annotation in annotation_rows(id, document, &mut problems, &mut refusals) {
        let Some((key, entity)) = entity(id, annotation, &mut problems, &mut refusals) else {
            continue;
        };
        if let Some((_, existing)) = entities
            .iter_mut()
            .find(|(k, e)| k == &key && e.kind == entity.kind)
        {
            existing.count += 1;
        } else {
            entities.push((key, entity));
        }
    }
    entities.sort_by(|(ka, a), (kb, b)| {
        b.count
            .cmp(&a.count)
            .then_with(|| order(a.kind).cmp(&order(b.kind)))
            .then_with(|| {
                a.name
                    .as_ref()
                    .map(Text::as_str)
                    .cmp(&b.name.as_ref().map(Text::as_str))
            })
            .then_with(|| ka.cmp(kb))
    });
    let total = u64::try_from(entities.len()).unwrap_or(u64::MAX);
    let value = if entities.is_empty() {
        Section::Empty {
            source: Source::PubTator,
            record: None,
        }
    } else {
        Section::Data {
            source: Source::PubTator,
            data: Capped {
                items: entities.into_iter().take(LIMIT).map(|(_, e)| e).collect(),
                total,
            },
        }
    };
    Mapped {
        refusals,
        value,
        problems,
    }
}
fn order(kind: AnnotationKind) -> u8 {
    match kind {
        AnnotationKind::Gene => 0,
        AnnotationKind::Variant => 1,
        AnnotationKind::Disease => 2,
        AnnotationKind::Chemical => 3,
    }
}

fn optional(
    id: Pmid,
    field: &str,
    raw: Option<&Field<String>>,
    problems: &mut Vec<Problem>,
) -> Option<Text> {
    let admit = if matches!(field, "name" | "text") {
        Text::from_source
    } else {
        Text::new
    };
    match raw {
        Some(Field::Value(s)) if s != "-" && !s.is_empty() => match admit(s) {
            Ok(text) => Some(text),
            Err(_) => {
                push_problem(problems, Source::PubTator, record_text(id), field);
                None
            }
        },
        Some(Field::Invalid(_)) => {
            push_problem(problems, Source::PubTator, record_text(id), field);
            None
        }
        _ => None,
    }
}
fn entity(
    id: Pmid,
    annotation: &Annotation,
    problems: &mut Vec<Problem>,
    refusals: &mut crate::Refusal,
) -> Option<(String, AnnotatedEntity)> {
    let raw = &annotation.infons;
    let kind = match raw.kind.as_str() {
        "Gene" => AnnotationKind::Gene,
        "Variant" => AnnotationKind::Variant,
        "Disease" => AnnotationKind::Disease,
        "Chemical" => AnnotationKind::Chemical,
        _ => return None,
    };
    let identifier = optional(id, "identifier", raw.identifier.as_ref(), problems);
    let text = optional(id, "text", annotation.text.as_ref(), problems);
    let name = optional(id, "name", raw.name.as_ref(), problems).or(text);
    let mut value = AnnotatedEntity {
        kind,
        name,
        count: 1,
        entrez_id: None,
        mesh: None,
        hgvs: None,
        rsid: None,
    };
    match kind {
        AnnotationKind::Gene => {
            value.entrez_id = identifier.as_ref().and_then(|s| s.as_str().parse().ok());
            if identifier.is_some() && value.entrez_id.is_none() {
                push_problem(problems, Source::PubTator, record_text(id), "entrez_id");
            }
        }
        AnnotationKind::Variant => {
            value.hgvs = optional(id, "hgvs", raw.hgvs.as_ref(), problems);
            value.rsid = optional(id, "rsid", raw.rsid.as_ref(), problems);
            if value.hgvs.is_none() && value.rsid.is_none() {
                push_problem(problems, Source::PubTator, record_text(id), "variant");
                *refusals = crate::Refusal::Refused;
                return None;
            }
            value.entrez_id = match &raw.gene_id {
                Some(Field::Value(n)) => biodata::NcbiGeneId::new(*n).ok(),
                _ => None,
            };
            if raw.gene_id.is_some() && value.entrez_id.is_none() {
                push_problem(problems, Source::PubTator, record_text(id), "gene_id");
            }
        }
        AnnotationKind::Disease | AnnotationKind::Chemical => value.mesh = identifier.clone(),
    }
    let checked_identifier = match kind {
        AnnotationKind::Gene => value.entrez_id.map(|id| id.to_string()),
        _ => identifier.map(|s| s.to_string()),
    };
    if checked_identifier.is_none()
        && value.name.is_none()
        && value.hgvs.is_none()
        && value.rsid.is_none()
    {
        *refusals = crate::Refusal::Refused;
        return None;
    }
    let key = checked_identifier
        .map(|id| format!("id:{id}"))
        .unwrap_or_else(|| {
            if value.hgvs.is_some() || value.rsid.is_some() {
                format!(
                    "variant:{:?}/{:?}/{:?}",
                    value.entrez_id, value.hgvs, value.rsid
                )
            } else {
                format!("name:{}", value.name.as_ref().map_or("", Text::as_str))
            }
        });
    Some((key, value))
}

fn annotation_rows<'a>(
    id: Pmid,
    document: Option<&'a Document>,
    problems: &mut Vec<Problem>,
    refusals: &mut crate::Refusal,
) -> Vec<&'a Annotation> {
    let mut annotations = Vec::new();
    if let Some(passages) = document.and_then(|d| d.passages.as_ref()) {
        if let Some(passages) = passages.as_ref() {
            for passage in passages {
                let Some(passage) = passage.as_ref() else {
                    push_problem(problems, Source::PubTator, record_text(id), "passages");
                    continue;
                };
                if let Some(items) = &passage.annotations {
                    if let Some(items) = items.as_ref() {
                        for item in items {
                            if let Some(item) = item.as_ref() {
                                annotations.push(item);
                            } else {
                                *refusals = crate::Refusal::Refused;
                                push_problem(
                                    problems,
                                    Source::PubTator,
                                    record_text(id),
                                    "annotations",
                                );
                            }
                        }
                    } else {
                        push_problem(problems, Source::PubTator, record_text(id), "annotations");
                    }
                }
            }
        } else {
            push_problem(problems, Source::PubTator, record_text(id), "passages");
        }
    }
    annotations
}
