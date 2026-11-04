//! Release projection requires a matching total in the same source response.
use biodata::{Attribution, Build, PopulationDataType, PopulationFrequency, Text, VariantRef};
use biomcp_adapters::gnomad::variant::{CohortFrequency, Subset, Variant};
use biomcp_types::Source;
use biomcp_types::{Problem, ProblemCode};

pub(super) fn frequency(
    raw: &Variant,
    subset: &Subset,
    data_type: PopulationDataType,
    va: Option<&Vec<CohortFrequency>>,
    name: &str,
    subject: Option<&VariantRef>,
    problems: &mut Vec<Problem>,
) -> Option<PopulationFrequency> {
    if subset.ac.is_none() || subset.an.is_none() || rejected_release(name, problems) {
        return None;
    }
    if va.is_some_and(|rows| {
        rows.iter().any(|row| {
            row.cohort.id.as_str() == "ALL"
                && (row.focus_allele_count.is_none() || row.locus_allele_count.is_none())
        })
    }) {
        return None;
    }
    let result = checked(subset, va, name, subject);
    let (version, subject) = match result {
        Ok(value) => value,
        Err(field) => {
            // Preserve the existing source-level missing-release problem once.
            if !problems.iter().any(|p| p.field == field) {
                problems.push(Problem {
                    source: Source::Gnomad,
                    record: Some(raw.variant_id.clone()),
                    field,
                    code: ProblemCode::ValueUnknown,
                });
            }
            return None;
        }
    };
    let dataset = Source::Gnomad.text();
    let frequency = PopulationFrequency::new(
        subject.clone(),
        Build::GRCh38,
        dataset,
        version.clone(),
        Attribution {
            approved: None,
            published: None,
            source: Source::Gnomad.text(),
            record: Some(raw.variant_id.clone()),
            release: Some(version.clone()),
            version: None,
            submitter: None,
            evaluated: None,
            url: None,
        },
    )
    .with_stratum(Some(data_type), None, None, None)
    .and_then(|frequency| frequency.with_counts(subset.ac, subset.an, None, None));
    match frequency {
        Ok(frequency) => Some(match &subset.filters {
            Some(filters) => frequency.with_filters(filters.clone().into()),
            None => frequency,
        }),
        Err(_) => {
            let field = format!("{name}.counts");
            if !problems
                .iter()
                .any(|problem| problem.field.as_ref() == field)
            {
                problems.push(Problem {
                    source: Source::Gnomad,
                    record: Some(raw.variant_id.clone()),
                    field: field.into(),
                    code: ProblemCode::ValueUnknown,
                });
            }
            None
        }
    }
}

fn checked<'a>(
    subset: &Subset,
    va: Option<&'a Vec<CohortFrequency>>,
    name: &str,
    subject: Option<&'a VariantRef>,
) -> Result<(Text, &'a VariantRef), Box<str>> {
    let rows = va.ok_or_else(|| Box::<str>::from("dataset_version"))?;
    let mut totals = rows.iter().filter(|v| v.cohort.id.as_str() == "ALL");
    let row = totals
        .next()
        .ok_or_else(|| Box::<str>::from("dataset_version"))?;
    if totals.next().is_some() {
        return Err(format!("va.{name}.cohort").into());
    }
    if (row.focus_allele_count, row.locus_allele_count) != (subset.ac, subset.an) {
        return Err(format!("va.{name}.counts").into());
    }
    let version = row
        .derived_from
        .as_ref()
        .and_then(|d| d.version.as_deref())
        .and_then(|version| Text::new(version).ok())
        .ok_or_else(|| Box::<str>::from("dataset_version"))?;
    let subject = subject.ok_or_else(|| Box::<str>::from("subject"))?;
    Ok((version, subject))
}

// A rejected release value already names the cause; do not add dependent absence.
fn rejected_release(name: &str, problems: &[Problem]) -> bool {
    let prefix = format!("va.{name}.derivedFrom");
    problems.iter().any(|problem| {
        problem.field.as_ref() == prefix || problem.field.as_ref() == format!("{prefix}.version")
    })
}

/// Bind the query dataset to a release actually reported for both matching source totals.
/// gnomAD publishes no independent VA joint release: retain joint values without one.
pub(super) fn shared_release(
    raw: &Variant,
    exome: Option<&biomcp_types::GnomadCounts>,
    genome: Option<&biomcp_types::GnomadCounts>,
    problems: &mut Vec<Problem>,
) -> Option<Text> {
    let exome = exome.and_then(|counts| counts.frequency.as_ref());
    let genome = genome.and_then(|counts| counts.frequency.as_ref());
    if let Some((exome, genome)) = exome.zip(genome)
        && exome.dataset_version() == genome.dataset_version()
    {
        return Some(exome.dataset_version().clone());
    }
    if raw.joint.is_some() {
        problems.push(Problem {
            source: Source::Gnomad,
            record: Some(raw.variant_id.clone()),
            field: "joint.dataset_version".into(),
            code: ProblemCode::ValueUnknown,
        });
    }
    None
}
