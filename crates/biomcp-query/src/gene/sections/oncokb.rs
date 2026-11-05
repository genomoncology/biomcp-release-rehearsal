use super::{SectionResult, SectionValue};
use crate::{Ctx, status};
use biodata::{Gene, HgncId};

use biomcp_adapters::oncokb;
use biomcp_fetch::{Deadline, SecretName};
use biomcp_types::{GeneSection, Reason, Section, SourceState};

pub(super) fn run(cx: &Ctx<'_>, deadline: Deadline<'_>, genes: &[(HgncId, Gene)]) -> SectionResult {
    if cx.fetch.disabled(GeneSection::Oncokb.source()) {
        return skipped(genes, Reason::Disabled);
    }
    if !cx.fetch.has_key(SecretName::OncokbApiKey) {
        return skipped(genes, Reason::NoKey);
    }
    let leg = cx.fetch.leg(&oncokb::SPEC, deadline).fresh(cx.fresh);
    let mut error = None;
    let mut local = None;
    let mut problems = Vec::new();
    let mut values = Vec::new();
    for (id, gene) in genes {
        let result = gene
            .entrez_id
            .map(|entrez| oncokb::gene(&leg, &gene.symbol, entrez));
        let (section, card, partial) = match result {
            Some(Ok(mapped)) => {
                let partial = mapped.refusals == biomcp_adapters::Refusal::Refused;
                status::append(&mut problems, mapped.problems);
                mapped_value(mapped.value, partial)
            }
            Some(Err(failure)) => {
                error = Some(failure);
                local = local.or(status::local(failure));
                (
                    Section::unavailable(
                        GeneSection::Oncokb.source(),
                        crate::status::reason(failure).unwrap_or(biomcp_types::Reason::Unavailable),
                    ),
                    None,
                    true,
                )
            }
            None => (Section::NotApplicable, None, false),
        };
        values.push((*id, SectionValue::Oncokb(section, card, partial)));
    }
    if values
        .iter()
        .any(|(_, value)| matches!(value, SectionValue::Oncokb(_, Some(_), _)))
    {
        match oncokb::version(&leg) {
            Ok(version) => {
                let partial_version = version.refusals == biomcp_adapters::Refusal::Refused;
                status::append(&mut problems, version.problems);
                apply_version(&mut values, version.value.as_ref(), partial_version);
            }
            Err(failure) => {
                error = Some(failure);
                local = local.or(status::local(failure));
                for (_, value) in &mut values {
                    if let SectionValue::Oncokb(_, Some(_), partial) = value {
                        *partial = true;
                    }
                }
            }
        }
    }
    let source = status::source(GeneSection::Oncokb.source(), leg.finish(), error);
    if source.state == SourceState::Stale {
        for (_, value) in &mut values {
            if let SectionValue::Oncokb(_, Some(_), partial) = value {
                *partial = true;
            }
        }
    }
    SectionResult {
        dropped: false,
        values,
        source,
        problems,
        local,
    }
}

fn skipped(genes: &[(HgncId, Gene)], reason: Reason) -> SectionResult {
    SectionResult {
        dropped: false,
        values: genes
            .iter()
            .map(|(id, _)| {
                (
                    *id,
                    SectionValue::Oncokb(
                        Section::Skipped {
                            source: GeneSection::Oncokb.source(),
                            reason,
                        },
                        None,
                        false,
                    ),
                )
            })
            .collect(),
        source: biomcp_adapters::registry::outcome(
            GeneSection::Oncokb.source(),
            biomcp_types::SourceStatus::Skipped(Some(reason)),
        ),
        problems: Vec::new(),
        local: None,
    }
}
fn mapped_value(
    data: Option<oncokb::GeneData>,
    partial: bool,
) -> (
    Section<biomcp_types::Oncokb>,
    Option<biomcp_types::OncokbCard>,
    bool,
) {
    match data {
        Some(data) => (
            Section::Data {
                source: GeneSection::Oncokb.source(),
                data: data.section,
            },
            Some(data.card),
            partial,
        ),
        None if partial => (
            Section::unavailable(GeneSection::Oncokb.source(), Reason::BadReply),
            None,
            true,
        ),
        None => (
            Section::Empty {
                source: GeneSection::Oncokb.source(),
                record: None,
            },
            None,
            false,
        ),
    }
}

fn apply_version(
    values: &mut [(HgncId, SectionValue)],
    version: Option<&biodata::Text>,
    partial_version: bool,
) {
    for (_, value) in values {
        if let SectionValue::Oncokb(section, Some(card), partial) = value {
            card.version = version.cloned();
            *partial |= partial_version;
            if let Section::Data { data, .. } = section {
                data.version = version.cloned();
            }
        }
    }
}
