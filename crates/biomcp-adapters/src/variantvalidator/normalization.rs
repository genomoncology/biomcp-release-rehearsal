//! Preserve optional native forms without dropping the normalization record.
use super::{
    Build, Leg, RequestError, Source, fetch,
    raw::{Loci, RawVariant},
};
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::SourceText;
use biomcp_types::{
    NormalizationField, NormalizationMessage, NormalizationResult, VariantResolution,
};

/// Keep each provider mapping and warning; never select a transcript.
pub fn normalize(
    leg: &Leg<'_>,
    input: &str,
    build: Build,
) -> Result<Mapped<NormalizationResult>, RequestError> {
    let mut raw = fetch(leg, input, build)?;
    let mut problems = Problems {
        source: Source::VariantValidator,
        record: None,
        list: Vec::new(),
    };
    let mut result = NormalizationResult::unavailable(Source::VariantValidator);
    result.outcome = if raw.flag.as_str() == "warning" {
        VariantResolution::Rejected
    } else {
        VariantResolution::Resolved
    };
    result.version = std::mem::take(&mut raw.metadata)
        .into_iter()
        .filter_map(|(field, raw)| {
            let value = problems.field(&format!("metadata.{field}"), Some(raw))?;
            Some(NormalizationField {
                field: field.into(),
                value,
            })
        })
        .collect();
    let mut mappings = Vec::new();
    let mut messages = Vec::new();
    let mut refusals = crate::Refusal::None;
    for (key, row) in raw.records(input, &mut problems, &mut refusals) {
        add_record(&key, row, &mut mappings, &mut messages, &mut problems);
    }
    result.total_mappings = mappings.len() as u32;
    result.mappings = mappings.into();
    result.messages = messages.into();
    Ok(Mapped {
        refusals,
        value: result,
        problems: problems.list,
    })
}
fn add_record(
    key: &str,
    row: RawVariant,
    mappings: &mut Vec<NormalizationField>,
    messages: &mut Vec<NormalizationMessage>,
    problems: &mut Problems,
) {
    problems.record = crate::mapped::record(key);
    for text in problems.fields(
        &format!("{key}.validation_warnings"),
        row.validation_warnings,
    ) {
        messages.push(NormalizationMessage {
            category: "validation_warnings".into(),
            code: None,
            text,
        });
    }
    for (field, value) in [
        (
            "hgvs_transcript_variant",
            Some(Field::Value(row.hgvs_transcript_variant.into())),
        ),
        (
            "hgvs_lrg_transcript_variant",
            row.hgvs_lrg_transcript_variant,
        ),
        ("hgvs_lrg_variant", row.hgvs_lrg_variant),
        ("hgvs_refseqgene_variant", row.hgvs_refseqgene_variant),
    ] {
        mapping(&format!("{key}.{field}"), value, mappings, problems);
    }
    let prefix = format!("{key}.hgvs_predicted_protein_consequence");
    if let Some(values) = problems.field(&prefix, row.hgvs_predicted_protein_consequence) {
        for (field, value) in values {
            mapping(
                &format!("{prefix}.{field}"),
                Some(value),
                mappings,
                problems,
            );
        }
    }
    loci(
        &format!("{key}.primary_assembly_loci"),
        row.primary_assembly_loci,
        mappings,
        problems,
    );
    let field = format!("{key}.alt_genomic_loci");
    for rows in problems.fields(&field, row.alt_genomic_loci) {
        loci(&field, Some(Field::Value(rows)), mappings, problems);
    }
}

fn mapping(
    field: &str,
    raw: Option<Field<SourceText>>,
    mappings: &mut Vec<NormalizationField>,
    problems: &mut Problems,
) {
    // The native service uses an empty HGVS form when that form is unavailable;
    // normalization has always omitted that sentinel before interpreting any form.
    if let Some(value) = problems
        .field(field, raw)
        .filter(|value| !value.as_str().is_empty())
    {
        mappings.push(NormalizationField {
            field: field.into(),
            value,
        });
    }
}
fn loci(
    field: &str,
    raw: Option<Field<Loci>>,
    mappings: &mut Vec<NormalizationField>,
    problems: &mut Problems,
) {
    if let Some(rows) = problems.field(field, raw) {
        for (assembly, raw) in rows {
            let field = format!("{field}.{assembly}");
            let Some(row) = problems.field(&field, Some(raw)) else {
                continue;
            };
            if let Some(value) = problems.field(&field, Some(row.hgvs_genomic_description)) {
                mappings.push(NormalizationField {
                    field: field.into(),
                    value: value.to_string().into(),
                });
            }
        }
    }
}
