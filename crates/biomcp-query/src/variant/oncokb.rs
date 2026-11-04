//! Missing keys and disabled policy stop before source, cache and replay access.
use crate::{Ctx, status};
use biodata::Build;
use biomcp_adapters::oncokb;
use biomcp_fetch::{Deadline, SecretName};
use biomcp_types::{
    PageInfo, Reason, Section, Source, SourceOutcome, SourceState, VariantGet, VariantHeader,
    VariantOncokb, VariantSection,
};

pub(super) fn section(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &[VariantHeader],
    deadline: Deadline<'_>,
) -> super::sections::Result<VariantOncokb> {
    if cx.fetch.disabled(Source::Oncokb) {
        return skipped(Reason::Disabled);
    }
    if !cx.fetch.has_key(SecretName::OncokbApiKey) {
        return skipped(Reason::NoKey);
    }
    let (Some(build), Some(tumor)) = (query.build(), query.tumor_type()) else {
        return skipped(Reason::NoJoinId);
    };
    let Some(hgvsg) = candidates
        .first()
        .and_then(|row| genomic(row, build))
        .or_else(|| input_genomic(query, build))
    else {
        return unavailable(Reason::NoJoinId);
    };
    let leg = cx.fetch.leg(&oncokb::SPEC, deadline).fresh(cx.fresh);
    let reply = oncokb::variant::annotation(&leg, &hgvsg, build, tumor.as_str());
    let error = reply.as_ref().err().copied();
    let source = status::source(Source::Oncokb, leg.finish(), error);
    let mut dropped = false;
    let mut problems = Vec::new();
    let section = match reply {
        Ok(row) => {
            let mapped = oncokb::variant_map::records(row, tumor.clone());
            dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
            problems = mapped.problems;
            let mut data = mapped.value;
            let total = data.page.total;
            data.records = std::mem::take(&mut data.records)
                .into_vec()
                .into_iter()
                .skip(query.section_offset() as usize)
                .take(query.section_limit() as usize)
                .collect();
            let returned = data.records.len() as u32;
            let next_offset = u64::from(query.section_offset()) + u64::from(returned);
            data.page = PageInfo {
                offset: query.section_offset(),
                returned,
                total,
                pool: None,
                truncated: None,
            };
            data.next = if next_offset < total {
                Some(
                    query
                        .at_section_offset(next_offset as u32)
                        .with_sections(vec![VariantSection::Oncokb].into()),
                )
            } else {
                None
            };
            Section::Data {
                source: Source::Oncokb,
                data,
            }
        }
        Err(error) => Section::unavailable(
            Source::Oncokb,
            crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
        ),
    };
    (
        section,
        vec![source],
        problems,
        error.and_then(status::local),
        dropped,
    )
}
fn genomic(row: &VariantHeader, build: Build) -> Option<String> {
    row.identity.representations.iter().find_map(|r| {
        if r.build != Some(build) || r.expression.syntax() != biodata::ExpressionSyntax::HgvsG {
            return None;
        }
        let (reference, body) = r.expression.value().as_str().split_once(':')?;
        let chromosome = if let Some(allele) = &r.allele {
            biodata::Chromosome::from_accession(allele.location().sequence, build).ok()?
        } else {
            reference
                .strip_prefix("chr")
                .unwrap_or(reference)
                .parse()
                .ok()?
        };
        Some(format!("{}:{body}", chromosome.name()))
    })
}
fn skipped(reason: Reason) -> super::sections::Result<VariantOncokb> {
    (
        Section::Skipped {
            source: Source::Oncokb,
            reason,
        },
        vec![source(SourceState::Skipped, reason)],
        Vec::new(),
        None,
        false,
    )
}
fn unavailable(reason: Reason) -> super::sections::Result<VariantOncokb> {
    (
        Section::unavailable(Source::Oncokb, reason),
        vec![source(SourceState::Failed, reason)],
        Vec::new(),
        None,
        false,
    )
}
fn source(state: SourceState, reason: Reason) -> SourceOutcome {
    biomcp_adapters::registry::outcome(Source::Oncokb, state, Some(reason))
}

pub(super) fn input_genomic(query: &VariantGet, build: Build) -> Option<String> {
    let (reference, body) = query.input().as_str().split_once(':')?;
    if !body.starts_with("g.") {
        return None;
    }
    let chromosome = if let Ok(accession) = reference.parse::<biodata::Accession>() {
        biodata::Chromosome::from_accession(accession, build).ok()?
    } else {
        reference
            .strip_prefix("chr")
            .unwrap_or(reference)
            .parse()
            .ok()?
    };
    Some(format!("{}:{body}", chromosome.name()))
}

/// Offer setup only for the missing caller context; never supply a default.
pub(super) fn next(query: &VariantGet, sources: &[SourceOutcome]) -> Vec<crate::next::Step> {
    if sources
        .iter()
        .any(|s| s.source == Source::Oncokb && s.reason == Some(Reason::NoKey))
    {
        vec![crate::next::Step::new(
            biomcp_types::StepKind::Key,
            &[biomcp_types::Source::Oncokb],
            crate::Action::KeyHelp(biomcp_fetch::SecretName::OncokbApiKey),
        )]
    } else if (query.build().is_none() || query.tumor_type().is_none())
        && sources
            .iter()
            .any(|s| s.source == Source::Oncokb && s.reason == Some(Reason::NoJoinId))
    {
        vec![crate::next::Step::new(
            biomcp_types::StepKind::Configure,
            &[],
            crate::Action::QueryHelp(biomcp_types::Request::GetVariant(query.clone())),
        )]
    } else {
        Vec::new()
    }
}
