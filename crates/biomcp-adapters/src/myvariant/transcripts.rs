//! Preserve only source-paired transcript rows; independent annotation arrays are not joined.
use crate::mapped::Problems;
use crate::raw::{Field, OneOrMany};
use biodata::Text;
use biomcp_types::{Problem, Source, VariantTranscript};
use serde::Deserialize;

pub(super) const FIELD: &str = "snpeff.ann";

#[derive(Deserialize)]
pub(super) struct Snpeff {
    ann: Option<Field<OneOrMany<Field<Annotation>>>>,
}
#[derive(Deserialize)]
struct Annotation {
    feature_id: Text,
    feature_type: Text,
    genename: Option<Field<Text>>,
    hgvs_c: Option<Field<Text>>,
    hgvs_p: Option<Field<Text>>,
}
pub(super) fn map(
    raw: Option<Field<Snpeff>>,
    record: &Text,
    problems: &mut Vec<Problem>,
) -> Box<[VariantTranscript]> {
    let mut fields = Problems {
        source: Source::MyVariant,
        record: Some(record.clone()),
        list: Vec::new(),
    };
    let rows = map_rows(raw, &mut fields);
    for problem in fields.list {
        crate::mapped::push_problem(problems, problem.source, problem.record, &problem.field);
    }
    rows
}
fn map_rows(raw: Option<Field<Snpeff>>, problems: &mut Problems) -> Box<[VariantTranscript]> {
    let Some(raw) = problems.field(FIELD, raw) else {
        return Box::default();
    };
    let Some(ann) = problems.field(FIELD, raw.ann) else {
        return Box::default();
    };
    let Some(component) = problems.text(FIELD, Some(FIELD)) else {
        return Box::default();
    };
    let mut rows = Vec::new();
    for annotation in ann.values() {
        let Some(a) = problems.field(FIELD, Some(annotation)) else {
            continue;
        };
        if a.feature_type.as_str() != "transcript" {
            continue;
        }
        let gene = problems.field("snpeff.ann.genename", a.genename);
        let hgvs_c = notation(problems, "snpeff.ann.hgvs_c", a.hgvs_c, &["c.", "n.", "r."]);
        let hgvs_p = notation(problems, "snpeff.ann.hgvs_p", a.hgvs_p, &["p."]);
        rows.push(VariantTranscript {
            source: Source::MyVariant,
            component: component.clone(),
            transcript: a.feature_id,
            gene,
            hgvs_c,
            hgvs_p,
            matched: false,
        });
    }
    rows.into()
}
fn notation(
    problems: &mut Problems,
    field: &str,
    raw: Option<Field<Text>>,
    prefixes: &[&str],
) -> Option<Text> {
    let value = problems.field(field, raw)?;
    problems.required(
        field,
        prefixes
            .iter()
            .any(|prefix| value.as_str().starts_with(prefix))
            .then_some(value),
    )
}

pub(super) fn label() -> Option<biomcp_types::ComponentLabel> {
    let declaration = super::components::SNPEFF;
    Text::new(declaration.name)
        .ok()
        .zip(super::components::label(declaration.terms).ok())
        .map(|(component, terms)| biomcp_types::ComponentLabel { component, terms })
}
