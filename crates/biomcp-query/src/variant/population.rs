//! Exact GRCh38 source joins and bounded population observations.
use crate::{Ctx, status};
use biodata::{Build, Chromosome, PopulationDataType, PopulationSex, Text};

use biomcp_adapters::gnomad;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    GnomadCounts, Problem, ProblemCode, Reason, Section, Source, SourceOutcome, VariantGet,
    VariantHeader, VariantPopulation,
};
/// Read only gnomAD after one unambiguous source allele has been established.
pub(super) fn section(
    cx: &Ctx<'_>,
    _query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: Deadline<'_>,
    blocked: Option<Reason>,
) -> super::sections::Result<VariantPopulation> {
    if let Some(reason) = blocked {
        return (
            Section::unavailable(Source::Gnomad, reason),
            Vec::new(),
            Vec::new(),
            None,
            false,
        );
    }
    let joined = super::population_identity::read(cx, candidates, deadline);
    let Some(id) = joined.id else {
        let mut result = missing(joined.reason.unwrap_or(Reason::NoJoinId));
        result.1 = joined.sources;
        result.2.extend(joined.problems);
        result.3 = joined.local;
        return result;
    };
    let deadline = deadline.after_stage(joined.elapsed);
    let leg = cx.fetch.leg(&gnomad::SPEC, deadline).fresh(cx.fresh);
    let mut dropped = false;
    let mut problems = joined.problems;
    let result = gnomad::variant::population(&leg, &id).and_then(|raw| {
        dropped |= raw.refusals == biomcp_adapters::Refusal::Refused;
        problems.extend(raw.problems);
        raw.value
            .map(|raw| summary(&raw, candidates, &mut problems))
            .transpose()
    });
    let error = result.as_ref().err().copied();
    let source = status::source(Source::Gnomad, leg.finish(), error);
    let value = match result {
        Ok(Some(data)) => Section::Data {
            source: Source::Gnomad,
            data,
        },
        Ok(None) => Section::Empty {
            source: Source::Gnomad,
            record: Text::new(&id).ok().map(|id| biomcp_types::SourceRecord {
                namespace: biodata::IdentifierNamespace::Unknown,
                id,
                url: None,
            }),
        },
        Err(e) => Section::unavailable(
            Source::Gnomad,
            crate::status::reason(e).unwrap_or(biomcp_types::Reason::Unavailable),
        ),
    };
    (
        value,
        joined.sources.into_iter().chain([source]).collect(),
        problems,
        joined.local.or(error.and_then(status::local)),
        dropped,
    )
}
/// Derive an exact SNV key without a second registry request.
pub(super) fn join(candidates: &[VariantHeader]) -> Option<String> {
    let [row] = candidates else {
        return None;
    };
    let mut forms = std::collections::BTreeSet::new();
    for representation in &row.identity.representations {
        if representation.build != Some(Build::GRCh38) {
            continue;
        }
        let Some(allele) = &representation.allele else {
            continue;
        };
        let chromosome =
            Chromosome::from_accession(allele.location().sequence, Build::GRCh38).ok()?;
        let reference = allele.reference().to_string();
        let alternate = allele.alternate().to_string();
        if allele.location().span.end() - allele.location().span.start() != 1
            || reference.len() != 1
            || alternate.len() != 1
            || !reference
                .bytes()
                .chain(alternate.bytes())
                .all(|b| b"ACGT".contains(&b))
        {
            continue;
        }
        forms.insert(format!(
            "{}-{}-{reference}-{alternate}",
            chromosome.name(),
            allele.location().span.start() + 1
        ));
    }
    if forms.len() != 1 {
        return None;
    }
    forms.into_iter().next()
}

pub(super) fn summary(
    raw: &gnomad::variant::Variant,
    candidates: &[VariantHeader],
    problems: &mut Vec<Problem>,
) -> Result<VariantPopulation, RequestError> {
    let subject = candidates
        .first()
        .and_then(|row| row.identity.ids.first())
        .cloned()
        .map(biodata::VariantRef::allele);
    let mut project = |subset: &gnomad::variant::Subset,
                       data_type,
                       va: Option<&Vec<gnomad::variant::CohortFrequency>>,
                       name| {
        let mut native = counts(subset, data_type, None);
        native.frequency = super::population_release::frequency(
            raw,
            subset,
            data_type,
            va,
            name,
            subject.as_ref(),
            problems,
        );
        Box::new(native)
    };
    let exome = raw.exome.as_ref().map(|s| {
        project(
            s,
            PopulationDataType::Exomes,
            raw.va.as_ref().and_then(|v| v.exome.as_ref()),
            "exome",
        )
    });
    let genome = raw.genome.as_ref().map(|s| {
        project(
            s,
            PopulationDataType::Genomes,
            raw.va.as_ref().and_then(|v| v.genome.as_ref()),
            "genome",
        )
    });
    let dataset_version = super::population_release::shared_release(
        raw,
        exome.as_deref(),
        genome.as_deref(),
        problems,
    );
    Ok(VariantPopulation {
        variant: raw.variant_id.clone(),
        dataset: text(gnomad::variant::DATASET)?,
        build: Build::GRCh38,
        dataset_version,
        exome,
        genome,
        joint: joint(raw, problems),
    })
}
fn joint(raw: &gnomad::variant::Variant, problems: &[Problem]) -> Section<Box<GnomadCounts>> {
    match &raw.joint {
        Some(subset) => Section::Data {
            source: Source::Gnomad,
            data: Box::new(counts(subset, PopulationDataType::Joint, None)),
        },
        None if problems.iter().any(|p| p.field.as_ref() == "joint") => {
            Section::unavailable(Source::Gnomad, Reason::BadReply)
        }
        None => Section::Empty {
            source: Source::Gnomad,
            record: None,
        },
    }
}
pub(super) fn counts(
    raw: &gnomad::variant::Subset,
    data_type: PopulationDataType,
    group: Option<&gnomad::variant::Group>,
) -> GnomadCounts {
    let (ancestry_group, sex) = group.map_or((None, None), |group| stratum(&group.id));
    GnomadCounts {
        frequency: None,
        data_type,
        group_id: group.map(|g| g.id.clone()),
        ancestry_group,
        sex,
        ac: group.map_or(raw.ac, |g| g.ac),
        an: group.map_or(raw.an, |g| g.an),
        homozygote_count: if group.is_none() {
            raw.homozygote_count
        } else {
            None
        },
        filters: raw.filters.clone().map(Into::into),
    }
}
fn stratum(id: &Text) -> (Option<Text>, Option<PopulationSex>) {
    let (ancestry, sex) = match id.as_str() {
        "XX" => ("", Some(PopulationSex::Xx)),
        "XY" => ("", Some(PopulationSex::Xy)),
        id => id
            .strip_suffix("_XX")
            .map(|a| (a, Some(PopulationSex::Xx)))
            .or_else(|| id.strip_suffix("_XY").map(|a| (a, Some(PopulationSex::Xy))))
            .unwrap_or((id, None)),
    };
    if ancestry.is_empty() {
        return (None, sex);
    }
    // Source PopulationId keys and PopulationIdAndChromosome grammar:
    // https://github.com/broadinstitute/gnomad-browser/blob/db245371f1e572106bda8481ef0f9a6baac6c00d/dataset-metadata/gnomadPopulations.ts
    if [
        "afr",
        "ami",
        "amr",
        "asj",
        "eas",
        "mid",
        "eur",
        "nfe",
        "fin",
        "oth",
        "sas",
        "rmi",
        "remaining",
        "eas_jpn",
        "eas_kor",
        "eas_oea",
        "nfe_bgr",
        "nfe_est",
        "nfe_nwe",
        "nfe_onf",
        "nfe_seu",
        "nfe_swe",
    ]
    .contains(&ancestry)
    {
        (Text::new(ancestry).ok(), sex)
    } else {
        (None, None)
    }
}

fn missing(
    reason: Reason,
) -> (
    Section<VariantPopulation>,
    Vec<SourceOutcome>,
    Vec<Problem>,
    Option<RequestError>,
    bool,
) {
    (
        Section::Unavailable {
            source: Source::Gnomad,
            reason,
        },
        Vec::new(),
        if reason == Reason::NoJoinId {
            vec![Problem {
                source: Source::Gnomad,
                record: None,
                field: "GRCh38_allele".into(),
                code: ProblemCode::ValueUnknown,
            }]
        } else {
            Vec::new()
        },
        None,
        false,
    )
}
fn text(value: &str) -> Result<Text, RequestError> {
    Text::new(value).map_err(|_| {
        biomcp_fetch::SourceError {
            source: Source::Gnomad,
            reason: Reason::BadReply,
            status: None,
            tls_cause: None,
            retry_after: None,
        }
        .into()
    })
}
