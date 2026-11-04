use super::raw;
use biodata::{MaudeDevice, MaudeNarrative, MaudeReport};
pub(super) fn device(row: raw::Device, p: &mut crate::mapped::Problems) -> MaudeDevice {
    MaudeDevice {
        device_sequence_number: p.field("device_sequence_number", row.device_sequence_number),
        brand_name: p.field("brand_name", row.brand_name),
        generic_name: p.field("generic_name", row.generic_name),
        manufacturer_d_name: p.field("manufacturer_d_name", row.manufacturer_d_name),
        device_report_product_code: p
            .field("device_report_product_code", row.device_report_product_code),
        model_number: p.field("model_number", row.model_number),
        catalog_number: p.field("catalog_number", row.catalog_number),
        udi_di: p.field("udi_di", row.udi_di),
        udi_public: p.field("udi_public", row.udi_public),
        lot_number: p.field("lot_number", row.lot_number),
        device_operator: p.field("device_operator", row.device_operator),
        implant_flag: p.field("implant_flag", row.implant_flag),
        implant_date_year: p.field("implant_date_year", row.implant_date_year),
        date_removed_flag: p.field("date_removed_flag", row.date_removed_flag),
        date_removed_year: p.field("date_removed_year", row.date_removed_year),
        device_age_text: p.field("device_age_text", row.device_age_text),
        expiration_date_of_device: p
            .field("expiration_date_of_device", row.expiration_date_of_device),
        device_availability: p.field("device_availability", row.device_availability),
        device_evaluated_by_manufacturer: p.field(
            "device_evaluated_by_manufacturer",
            row.device_evaluated_by_manufacturer,
        ),
        device_event_key: p.field("device_event_key", row.device_event_key),
        date_received: p.field("date_received", row.date_received),
        other_id_number: p.field("other_id_number", row.other_id_number),
        combination_product_flag: p.field("combination_product_flag", row.combination_product_flag),
        serviced_by_3rd_party_flag: p
            .field("serviced_by_3rd_party_flag", row.serviced_by_3rd_party_flag),
    }
}
pub(super) fn narrative(row: raw::Narrative, p: &mut crate::mapped::Problems) -> MaudeNarrative {
    MaudeNarrative {
        mdr_text_key: p.field("mdr_text_key", row.mdr_text_key),
        patient_sequence_number: p.field("patient_sequence_number", row.patient_sequence_number),
        text_type_code: p.field("text_type_code", row.text_type_code),
        text: p.field("text", row.text),
    }
}
pub(super) fn report(row: raw::Report) -> crate::Mapped<super::Report> {
    let mut p = crate::mapped::Problems {
        source: biomcp_types::Source::Maude,
        record: crate::mapped::record(&row.mdr_report_key),
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let (devices, device_positions) = p.indexed_fields("device", row.devices, &mut refusals);
    let (narrative_rows, narrative_positions) =
        p.indexed_fields("mdr_text", row.narrative, &mut refusals);
    let value = MaudeReport {
        mdr_report_key: row.mdr_report_key,
        report_number: p.field("report_number", row.report_number),
        event_type: p.field("event_type", row.event_type),
        date_of_event: p.field("date_of_event", row.date_of_event),
        date_received: p.field("date_received", row.date_received),
        report_date: p.field("report_date", row.report_date),
        date_report: p.field("date_report", row.date_report),
        date_report_to_fda: p.field("date_report_to_fda", row.date_report_to_fda),
        number_devices_in_event: p.field("number_devices_in_event", row.number_devices_in_event),
        number_patients_in_event: p.field("number_patients_in_event", row.number_patients_in_event),
        devices: devices
            .into_vec()
            .into_iter()
            .map(|v| device(v, &mut p))
            .collect(),
        narrative: narrative_rows
            .into_vec()
            .into_iter()
            .map(|v| narrative(v, &mut p))
            .collect(),
    };
    crate::Mapped {
        refusals,
        value: super::Report {
            data: value,
            positions: super::Positions {
                devices: device_positions,
                narrative: narrative_positions,
            },
        },
        problems: p.list,
    }
}
