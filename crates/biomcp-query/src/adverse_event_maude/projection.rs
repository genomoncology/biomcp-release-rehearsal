use super::{AdverseEventGet, AdverseEventSection, Source, Text, rows, text};
use biomcp_adapters::{NativePositions, maude::Positions};
use biomcp_types::ReplyBody;
use biomcp_types::ReplyData;
use biomcp_types::{
    AdverseEventDevices, AdverseEventMaudeCard, AdverseEventNarrative, AdverseEventRecords,
};
/// Whole narrative page cap; oversized rows produce a failure and never fragments.
const NARRATIVE_PAGE_BYTES: usize = 32 * 1024;
pub(super) fn project(
    query: &AdverseEventGet,
    report: biodata::MaudeReport,
    snapshot: Option<Text>,
    positions: Positions,
) -> Result<(ReplyData, u64, u32, u32), biomcp_types::QueryInvalid> {
    let header = header(&report);
    let row = report;
    let offset = query.section_offset();
    let end = offset.saturating_add(query.section_limit());
    let result = match query.section() {
        Some(AdverseEventSection::Devices) => devices(query, &row, &positions, header, snapshot)?,
        Some(AdverseEventSection::Narrative) => {
            let total = positions
                .narrative
                .as_ref()
                .map_or(row.narrative.len() as u64, |p| u64::from(p.total));
            let end = end.min(total as u32);
            let candidates =
                NativePositions::enumerate(&row.narrative, positions.narrative.as_ref())
                    .filter(|(index, _)| *index >= offset && *index < end)
                    .map(|(i, r)| rows::narrative(i, r.clone()));
            let (records, end) = narrative_page(candidates, end)?;
            let returned = records.len() as u32;
            (
                (AdverseEventNarrative {
                    header,
                    snapshot,
                    narrative: AdverseEventRecords {
                        records,
                        offset: query.section_offset(),
                        total,
                    },
                    components: Box::default(),
                })
                .with_query(query.clone()),
                total,
                returned,
                end,
            )
        }
        _ => (
            (card(query, row, header, snapshot, &positions)?).with_query(query.clone()),
            0,
            0,
            0,
        ),
    };
    Ok(result)
}

fn devices(
    query: &AdverseEventGet,
    row: &biodata::MaudeReport,
    positions: &Positions,
    header: biomcp_types::AdverseEventHeader,
    snapshot: Option<Text>,
) -> Result<(ReplyData, u64, u32, u32), biomcp_types::QueryInvalid> {
    let offset = query.section_offset();
    let end = offset.saturating_add(query.section_limit());
    let total = positions
        .devices
        .as_ref()
        .map_or(row.devices.len() as u64, |p| u64::from(p.total));
    let end = end.min(total as u32);
    let records: Box<[_]> = NativePositions::enumerate(&row.devices, positions.devices.as_ref())
        .filter(|(index, _)| *index >= offset && *index < end)
        .map(|(i, r)| rows::device(i, r.clone()))
        .collect();
    let returned = records.len() as u32;
    Ok((
        (AdverseEventDevices {
            header,
            snapshot,
            devices: AdverseEventRecords {
                records,
                offset: query.section_offset(),
                total,
            },
            components: biomcp_adapters::adverse_event_sources::components::labels(
                Source::Maude,
                Source::Gmdn,
            )
            .map_err(|_| biomcp_types::QueryInvalid::default())?,
        })
        .with_query(query.clone()),
        total,
        returned,
        end,
    ))
}

fn card(
    query: &AdverseEventGet,
    row: biodata::MaudeReport,
    header: biomcp_types::AdverseEventHeader,
    snapshot: Option<Text>,
    positions: &Positions,
) -> Result<AdverseEventMaudeCard, biomcp_types::QueryInvalid> {
    let card = AdverseEventMaudeCard {
        header,
        snapshot,
        report_number: row.report_number,
        event_date: row.date_of_event,
        received: row.date_received,
        report_date: row.report_date,
        date_report: row.date_report,
        date_report_to_fda: row.date_report_to_fda,
        number_devices_in_event: row.number_devices_in_event,
        number_patients_in_event: row.number_patients_in_event,
        device_count: positions
            .devices
            .as_ref()
            .map_or(row.devices.len() as u32, |p| p.total),
        narrative_count: positions
            .narrative
            .as_ref()
            .map_or(row.narrative.len() as u32, |p| p.total),
        source_url: text("https://open.fda.gov/apis/device/event/")?,
        report_url: text(
            &biomcp_adapters::registry::records::record_url(
                biomcp_types::Source::Maude,
                biomcp_adapters::registry::records::RecordKind::AdverseEvent,
                query.id().as_str(),
            )
            .ok_or_else(biomcp_types::QueryInvalid::default)?,
        )?,
        available_sections: vec![AdverseEventSection::Devices, AdverseEventSection::Narrative]
            .into(),
        components: Box::default(),
    };
    Ok(card)
}

fn narrative_page(
    rows: impl Iterator<Item = biomcp_types::AdverseEventNarrativeItem>,
    mut end: u32,
) -> Result<(Box<[biomcp_types::AdverseEventNarrativeItem]>, u32), biomcp_types::QueryInvalid> {
    let mut records = Vec::new();
    let mut bytes = 2;
    for row in rows {
        let additional = serde_json::to_vec(&row)
            .map_err(|_| biomcp_types::QueryInvalid::default())?
            .len()
            + usize::from(!records.is_empty());
        if bytes + additional > NARRATIVE_PAGE_BYTES {
            if records.is_empty() {
                return Err(biomcp_types::QueryInvalid::default());
            }
            end = row.index;
            break;
        }
        bytes += additional;
        records.push(row);
    }
    Ok((records.into(), end))
}

fn header(row: &biodata::MaudeReport) -> biomcp_types::AdverseEventHeader {
    biomcp_types::AdverseEventHeader {
        id: row.mdr_report_key.clone(),
        source: biomcp_types::AdverseEventSource::Maude,
        version: None,
        serious: None,
        event_type: row.event_type.clone(),
        first_received: row.date_received.clone(),
    }
}
