use crate::Outcome;
use biodata::{FaersReport, Text};
use biomcp_adapters::{
    NativePositions,
    faers::{
        Positions,
        words::{PATIENT_SEX, REPORTER_QUALIFICATION, label},
    },
};
use biomcp_fetch::RequestError;
use biomcp_types::ReplyBody;
use biomcp_types::{
    AdverseEventGet, AdverseEventHeader, AdverseEventPatient, AdverseEventReporter,
    AdverseEventSection as S, AdverseEventSource, Request, StepKind,
};
mod identifiers;
mod rows;
pub(super) fn render(
    q: &AdverseEventGet,
    r: FaersReport,
    url: Text,
    snapshot: Option<Text>,
    positions: Positions,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let navigation = snapshot
        .clone()
        .map_or_else(|| q.clone(), |t| q.with_snapshot(t));
    let header = AdverseEventHeader {
        id: r.safetyreportid.clone(),
        source: AdverseEventSource::Faers,
        version: r.safetyreportversion.clone(),
        serious: r.serious.clone(),
        event_type: None,
        first_received: r.receivedate.clone(),
    };
    let components = if q.section() == Some(S::Reactions) {
        biomcp_adapters::adverse_event_sources::components::labels(
            biomcp_types::Source::Faers,
            biomcp_types::Source::Meddra,
        )?
    } else {
        Box::default()
    };
    match q.section() {
        None => {
            card(q, r, url, header, snapshot, &positions, out)?;
            for section in [S::Drugs, S::Reactions] {
                section_step(&navigation, section, None, out);
            }
        }
        Some(S::Drugs) => drugs(q, &r, &positions, header, snapshot, components, out),
        Some(S::Reactions) => {
            let total = positions
                .reactions
                .as_ref()
                .map_or(r.reactions.len() as u64, |p| u64::from(p.total));
            let records = rows::reactions(q, &r.reactions, positions.reactions.as_ref());
            page(
                &navigation,
                q.section_offset()
                    .saturating_add(q.section_limit())
                    .min(total as u32),
                total,
                out,
            );
            out.body = (biomcp_types::AdverseEventReactions {
                header,
                snapshot,
                reactions: biomcp_types::AdverseEventRecords {
                    records,
                    offset: q.section_offset(),
                    total,
                },
                components,
            })
            .with_query(q.clone());
        }
        Some(section) => {
            let drug = NativePositions::enumerate(&r.drugs, positions.drugs.as_ref())
                .find(|(index, _)| Some(*index) == q.drug_index())
                .map(|(_, row)| row);
            identifiers::render(q, drug, section, header, snapshot, components, out)?;
        }
    }
    Ok(())
}
fn drugs(
    q: &AdverseEventGet,
    r: &FaersReport,
    positions: &Positions,
    header: AdverseEventHeader,
    snapshot: Option<Text>,
    components: Box<[biomcp_types::ComponentLabel]>,
    out: &mut Outcome,
) {
    let navigation = snapshot
        .clone()
        .map_or_else(|| q.clone(), |t| q.with_snapshot(t));
    let total = positions
        .drugs
        .as_ref()
        .map_or(r.drugs.len() as u64, |p| u64::from(p.total));
    let records = rows::drugs(q, &r.drugs, positions.drugs.as_ref());
    page(
        &navigation,
        q.section_offset()
            .saturating_add(q.section_limit())
            .min(total as u32),
        total,
        out,
    );
    out.body = (biomcp_types::AdverseEventDrugs {
        header,
        snapshot,
        drugs: biomcp_types::AdverseEventRecords {
            records,
            offset: q.section_offset(),
            total,
        },
        components,
    })
    .with_query(q.clone());
    let initial = out.next.len();
    for (index, drug) in
        NativePositions::enumerate(&r.drugs, positions.drugs.as_ref()).filter(|(index, _)| {
            *index >= q.section_offset()
                && *index < q.section_offset().saturating_add(q.section_limit())
        })
    {
        if let Some(ids) = &drug.openfda {
            for section in identifiers::available(ids) {
                if out.next.len() < initial + 3 {
                    section_step(&navigation, section, Some(index), out);
                }
            }
        }
    }
}
fn section_step(q: &AdverseEventGet, s: S, index: Option<u32>, out: &mut Outcome) {
    if let Some(q) = crate::next::checked(
        q.with_section(s, index),
        biomcp_types::Source::Faers,
        &mut out.problems,
    ) {
        out.next.push(crate::next::Step::run(
            StepKind::Section,
            &[biomcp_types::Source::Faers],
            Request::GetAdverseEvent(q),
            false,
        ));
    }
}
fn page(q: &AdverseEventGet, offset: u32, total: u64, out: &mut Outcome) {
    if offset > q.section_offset() && u64::from(offset) < total {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[biomcp_types::Source::Faers],
            Request::GetAdverseEvent(q.at_section_offset(offset)),
            false,
        ));
    }
}

fn card(
    q: &AdverseEventGet,
    r: FaersReport,
    url: Text,
    header: AdverseEventHeader,
    snapshot: Option<Text>,
    positions: &Positions,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    out.body = (biomcp_types::AdverseEventFaersCard {
        header,
        snapshot,
        latest_received: r.receiptdate,
        latest_received_format: r.receiptdateformat,
        transmitted: r.transmissiondate,
        transmitted_format: r.transmissiondateformat,
        first_received_format: r.receivedateformat,
        seriousness: r.seriousness,
        patient: r.patient.map(|native| AdverseEventPatient {
            sex_label: label(PATIENT_SEX, native.sex.as_deref()),
            native,
        }),
        reporter: r.reporter.map(|native| AdverseEventReporter {
            qualification_label: label(REPORTER_QUALIFICATION, native.qualification.as_deref()),
            native,
        }),
        drug_count: positions
            .drugs
            .as_ref()
            .map_or(r.drugs.len() as u32, |p| p.total),
        reaction_count: positions
            .reactions
            .as_ref()
            .map_or(r.reactions.len() as u32, |p| p.total),
        source_url: Text::new("https://open.fda.gov/apis/drug/event/")
            .map_err(|_| biomcp_fetch::InputError::Serialization)?,
        report_url: url,
        available_sections: vec![S::Drugs, S::Reactions].into(),
        components: Box::default(),
    })
    .with_query(q.clone());
    Ok(())
}
