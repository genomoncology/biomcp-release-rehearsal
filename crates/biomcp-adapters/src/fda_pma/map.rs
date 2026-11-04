use biodata::DeviceSubmission;
use biomcp_types::{DiagnosticPmaHeader, DiagnosticPmaRegulatory};
/// Whole supported native header fields; no application-wide claim.
pub fn header(row: &DeviceSubmission) -> Result<DiagnosticPmaHeader, biomcp_fetch::RequestError> {
    let id = format!(
        "{}{}",
        row.pma_number,
        row.supplement_number.as_ref().map_or("", |s| s.as_str())
    );
    Ok(DiagnosticPmaHeader {
        pma_number: row.pma_number.clone(),
        supplement_number: row.supplement_number.clone(),
        trade_name: row.trade_name.clone(),
        applicant: row.applicant.clone(),
        decision_date: row.decision_date.clone(),
        decision_code: row.decision_code.clone(),
        url: format!("https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfPMA/pma.cfm?ID={id}")
            .parse()
            .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::FdaPma, Some(200)))?,
    })
}
/// Native selected-row dates, committee, product and supplement metadata.
pub fn regulatory(row: &DeviceSubmission) -> DiagnosticPmaRegulatory {
    DiagnosticPmaRegulatory {
        date_received: row.date_received.clone(),
        fed_reg_notice_date: row.fed_reg_notice_date.clone(),
        advisory_committee: row.advisory_committee.clone(),
        advisory_committee_description: row.advisory_committee_description.clone(),
        product_code: row.product_code.clone(),
        supplement_type: row.supplement_type.clone(),
        supplement_reason: row.supplement_reason.clone(),
    }
}
/// Existing label splitter, with its escaped-byte fragment budget.
pub fn fragments(row: &DeviceSubmission) -> Vec<Option<String>> {
    row.ao_statement.as_deref().map_or_else(
        || vec![None],
        |text| {
            if text.is_empty() {
                vec![Some(String::new())]
            } else {
                crate::text_parts::split(text, 6144)
                    .into_iter()
                    .map(Some)
                    .collect()
            }
        },
    )
}
