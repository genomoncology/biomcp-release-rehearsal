use super::{Record, raw};
use crate::mapped::{Problems, record as record_text};
use crate::{Mapped, raw::Field};
use biodata::{Attribution, Curie, DiseaseEndpoint, DiseaseRelation, DiseaseSynonym, Text};
use biomcp_fetch::{InputError, RequestError};
use biomcp_types::{
    ComponentLabel, ComponentTerms, DiseaseCard, DiseaseConcept, DiseaseOntology, DiseaseSection,
    Source,
};
pub(super) fn attribution(
    id: &Curie,
    iri: Option<Text>,
    ontology: DiseaseOntology,
) -> Result<Attribution, RequestError> {
    Ok(Attribution {
        approved: None,
        published: None,
        source: crate::mapped::text(ontology.as_str()).map_err(|_| InputError::Serialization)?,
        record: record_text(id),
        url: iri,
        version: None,
        release: None,
        submitter: None,
        evaluated: None,
    })
}

/// Reviewed component terms remain separate from transport service terms.
pub fn components(ontology: DiseaseOntology) -> Result<Box<[ComponentLabel]>, RequestError> {
    let component = crate::disease_sources::components::get(Source::Ols, ontology.as_str())
        .ok_or(InputError::Serialization)?;
    Ok(vec![ComponentLabel {
        component: crate::mapped::text(component.name)
            .map_err(|_| RequestError::Input(InputError::Serialization))?,
        terms: ComponentTerms {
            usage: component.terms.usage,
            name: crate::mapped::text(component.terms.name)
                .map_err(|_| RequestError::Input(InputError::Serialization))?,
            url: crate::mapped::text(component.terms.url)
                .map_err(|_| RequestError::Input(InputError::Serialization))?,
        },
    }]
    .into())
}
pub(super) fn record(term: raw::Term, ontology: DiseaseOntology) -> Result<Record, RequestError> {
    let mut problems = Problems {
        source: Source::Ols,
        record: record_text(&term.obo_id),
        list: Vec::new(),
    };
    let iri = problems.optional(
        "iri",
        term.iri.as_ref().map(|v| match v {
            Field::Value(v) => Ok(v.clone()),
            Field::Invalid(_) => Err(()),
        }),
    );
    let label = problems.optional(
        "label",
        term.label.as_ref().map(|v| match v {
            Field::Value(v) => Ok(v.clone().into()),
            Field::Invalid(_) => Err(()),
        }),
    );
    let attribution = attribution(&term.obo_id, iri, ontology)?;
    let synonyms = synonyms(&term, &attribution);
    let xrefs = xrefs(&term, &attribution);
    let mut sections = Vec::new();
    if !synonyms.value.is_empty() {
        sections.push(DiseaseSection::Synonyms);
    }
    if !xrefs.value.is_empty() {
        sections.push(DiseaseSection::Xrefs);
    }
    let consider = term.annotation.as_ref().and_then(|value| match value {
        Field::Value(annotation) => ids(
            array(
                annotation.consider.as_ref(),
                "annotation.consider",
                &mut problems,
            ),
            "annotation.consider",
            &mut problems,
        ),
        Field::Invalid(_) => {
            problems.required::<()>("annotation", None);
            None
        }
    });
    let replaced_by = ids(
        array(
            term.term_replaced_by.as_ref(),
            "term_replaced_by",
            &mut problems,
        ),
        "term_replaced_by",
        &mut problems,
    );
    let definition = definition(term.description.as_ref(), &mut problems);
    Ok(Record {
        card: DiseaseCard {
            requested_id: term.obo_id.clone(),
            returned_id: term.obo_id.clone(),
            alias: None,
            concept: DiseaseConcept {
                id: Some(term.obo_id),
                label,
            },
            definition,
            obsolete: problems.field("is_obsolete", term.is_obsolete),
            consider,
            replaced_by,
            attribution,
            sections: sections.into(),
            components: components(ontology)?,
        },
        synonyms,
        xrefs,
        problems: problems.list,
    })
}
fn definition(
    values: Option<&Field<Vec<Field<crate::native_text::NativeText>>>>,
    problems: &mut Problems,
) -> Option<Text> {
    let descriptions: Vec<_> = array(values, "description", problems)
        .iter()
        .filter_map(|value| match value {
            Field::Value(v) => Some(v.as_str()),
            Field::Invalid(_) => {
                problems.required::<()>("description", None);
                None
            }
        })
        .collect();
    if descriptions.is_empty() {
        None
    } else {
        problems.text("description", Some(&descriptions.join("\n\n")))
    }
}
fn array<'a, T>(value: Option<&'a Field<Vec<T>>>, field: &str, problems: &mut Problems) -> &'a [T] {
    match value {
        Some(Field::Value(values)) => values,
        Some(Field::Invalid(_)) => {
            problems.required::<()>(field, None);
            &[]
        }
        None => &[],
    }
}
fn ids(values: &[Field<String>], field: &str, problems: &mut Problems) -> Option<Box<[Curie]>> {
    if values.is_empty() {
        return None;
    }
    Some(
        values
            .iter()
            .filter_map(|v| {
                problems.required(
                    field,
                    match v {
                        Field::Value(v) => iri_id(v.as_str()),
                        Field::Invalid(_) => None,
                    },
                )
            })
            .collect(),
    )
}
fn iri_id(value: &str) -> Option<Curie> {
    if let Some(native) = value.strip_prefix("http://purl.obolibrary.org/obo/") {
        let (prefix, local) = native.split_once('_')?;
        format!("{prefix}:{local}").parse().ok()
    } else {
        value.parse().ok()
    }
}

fn synonyms(term: &raw::Term, attribution: &Attribution) -> Mapped<Vec<DiseaseSynonym>> {
    let mut problems = Problems {
        source: Source::Ols,
        record: record_text(&term.obo_id),
        list: Vec::new(),
    };
    let mut native_scopes = Vec::new();
    for native in array(term.obo_synonym.as_ref(), "obo_synonym", &mut problems) {
        let Field::Value(native) = native else {
            problems.required::<()>("obo_synonym", None);
            continue;
        };
        let name = problems.required(
            "synonym.name",
            match &native.name {
                Field::Value(name) => Some(name),
                Field::Invalid(_) => None,
            },
        );
        let scope = problems.optional(
            "synonym.scope",
            native.scope.as_ref().map(|s| match s {
                Field::Value(s) => format!("oboInOwl:{}", s.as_str())
                    .parse::<Curie>()
                    .map_err(|_| ()),
                Field::Invalid(_) => Err(()),
            }),
        );
        if let Some(name) = name {
            native_scopes.push((name, scope));
        }
    }
    let mut value = Vec::new();
    for name in array(term.synonyms.as_ref(), "synonyms", &mut problems) {
        let name = problems.required(
            "synonyms",
            match name {
                Field::Value(name) => Some(name),
                Field::Invalid(_) => None,
            },
        );
        if let Some(name) = name {
            value.push(DiseaseSynonym {
                value: name.clone().into(),
                scope: native_scopes
                    .iter()
                    .find(|(n, _)| *n == name)
                    .and_then(|(_, s)| s.clone()),
                language: None,
                attribution: attribution.clone(),
            });
        }
    }
    Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    }
}
fn xrefs(term: &raw::Term, attribution: &Attribution) -> Mapped<Vec<DiseaseRelation>> {
    let mut mapped = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    let mut metadata = Problems {
        source: Source::Ols,
        record: record_text(&term.obo_id),
        list: Vec::new(),
    };
    for native in array(term.obo_xref.as_ref(), "obo_xref", &mut metadata) {
        let Field::Value(native) = native else {
            metadata.required::<()>("obo_xref", None);
            continue;
        };
        let mut problems = Problems {
            source: Source::Ols,
            record: record_text(&term.obo_id),
            list: Vec::new(),
        };
        let id = problems.required(
            "xref.id",
            match (&native.database, &native.id) {
                (Field::Value(database), Field::Value(id)) => {
                    format!("{database}:{id}").parse().ok()
                }
                _ => None,
            },
        );
        let description = problems.optional(
            "xref.description",
            native.description.as_ref().map(|v| match v {
                Field::Value(v) => Ok::<_, ()>(v.as_str().into()),
                Field::Invalid(_) => Err(()),
            }),
        );
        mapped.problems.extend(problems.list);
        let Some(id) = id else {
            continue;
        };
        let Ok(predicate) = "oboInOwl:hasDbXref".parse() else {
            continue;
        };
        mapped.value.push(DiseaseRelation {
            subject: DiseaseEndpoint {
                id: term.obo_id.clone(),
                label: match &term.label {
                    Some(Field::Value(v)) => Some(v.clone().into()),
                    _ => None,
                },
            },
            predicate,
            object: DiseaseEndpoint { id, label: None },
            attribution: attribution.clone(),
            description,
        });
    }
    mapped.problems.extend(metadata.list);
    mapped
}
