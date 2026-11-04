//! Preserve independently mapped native descriptions and source messages.
use super::{Leg, RequestError, Source, fetch};
use crate::{Mapped, mapped::Problems, raw::Field};
use biomcp_types::{
    NormalizationField, NormalizationMessage, NormalizationResult, VariantResolution,
};
/// Preserve source descriptions and messages without deriving a local equivalence.
pub fn normalize(leg: &Leg<'_>, input: &str) -> Result<Mapped<NormalizationResult>, RequestError> {
    let raw = fetch(leg, input)?;
    let mut problems = Problems {
        source: Source::Mutalyzer,
        record: crate::mapped::record(input),
        list: Vec::new(),
    };
    let mut result = NormalizationResult::unavailable(Source::Mutalyzer);
    let normalized = match raw.identity(input) {
        Ok(value) => value,
        Err(field) => {
            problems.required::<()>(field, None);
            return Ok(Mapped {
                refusals: crate::Refusal::Refused,
                value: result,
                problems: problems.list,
            });
        }
    };
    result.outcome = if raw.errors().is_empty() {
        VariantResolution::Resolved
    } else {
        VariantResolution::Rejected
    };
    result.normalized = normalized.map(|value| value.as_str().into());
    result.corrected = problems.field("corrected_description", raw.corrected_description);
    let mut mappings = Vec::new();
    for (field, raw) in [
        ("protein.description", raw.protein),
        ("rna.description", raw.rna),
    ] {
        if let Some(value) = problems
            .field(field, raw)
            .and_then(|row| problems.field(field, Some(row.description)))
        {
            mappings.push(NormalizationField {
                field: field.into(),
                value,
            });
        }
    }
    if let Some(groups) = problems.field("equivalent_descriptions", raw.equivalent_descriptions) {
        for (kind, rows) in groups {
            let field = format!("equivalent_descriptions.{kind}");
            for row in problems.fields(&field, Some(rows)) {
                if let Some(value) = problems.field(&field, Some(row.description)) {
                    mappings.push(NormalizationField {
                        field: field.clone().into(),
                        value,
                    });
                }
            }
        }
    }
    for value in problems.fields(
        "back_translated_descriptions",
        raw.back_translated_descriptions,
    ) {
        mappings.push(NormalizationField {
            field: "back_translated_descriptions".into(),
            value,
        });
    }
    result.total_mappings = mappings.len() as u32;
    result.mappings = mappings.into();
    result.messages = messages(raw.errors, raw.warnings, raw.infos, &mut problems);
    if let Some(value) = problems.field("version", raw.version) {
        result.version = vec![NormalizationField {
            field: "version".into(),
            value,
        }]
        .into();
    }
    Ok(Mapped {
        refusals: crate::Refusal::None,
        value: result,
        problems: problems.list,
    })
}
fn messages(
    errors: Option<Vec<super::Message>>,
    warnings: Option<Field<Vec<Field<super::Message>>>>,
    infos: Option<Field<Vec<Field<super::Message>>>>,
    problems: &mut Problems,
) -> Box<[NormalizationMessage]> {
    let mut messages = Vec::new();
    for (category, rows) in [
        ("errors", errors.unwrap_or_default().into_boxed_slice()),
        ("warnings", problems.fields("warnings", warnings)),
        ("infos", problems.fields("infos", infos)),
    ] {
        for row in rows {
            let text = match row.details {
                None => "".into(),
                Some(raw) => {
                    let Some(text) = problems.field(&format!("{category}.details"), Some(raw))
                    else {
                        continue;
                    };
                    text
                }
            };
            messages.push(NormalizationMessage {
                category: category.into(),
                code: Some(row.code.to_string().into()),
                text,
            });
        }
    }
    messages.into()
}
