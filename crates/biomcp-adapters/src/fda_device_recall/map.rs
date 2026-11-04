use super::raw;
use crate::{
    mapped::{Mapped, field_text, push_problem, record},
    raw::Field,
};
use biodata::Text;
use biomcp_types::{DeviceRecallCard, DeviceRecallHeader, Problem, Source};
const SOURCE: Source = Source::FdaDeviceRecall;
fn list(
    value: Option<Field<Vec<Field<Text>>>>,
    field: &str,
    id: Option<Text>,
    problems: &mut Vec<Problem>,
) -> Option<Box<[Text]>> {
    let values = match value? {
        Field::Value(values) => values,
        Field::Invalid(_) => {
            push_problem(problems, SOURCE, id, field);
            return None;
        }
    };
    let values: Box<[_]> = values
        .into_iter()
        .filter_map(|v| {
            let value = v.value();
            if value.is_none() {
                push_problem(problems, SOURCE, id.clone(), field);
            }
            value
        })
        .collect();
    (!values.is_empty()).then_some(values)
}
pub(super) fn mapped(row: raw::Record) -> Mapped<DeviceRecallCard> {
    let mut problems = Vec::new();
    let id = record(&row.product_res_number);
    let initiated = row
        .event_date_initiated
        .map(|v| v.value().filter(|d| d.day().is_some()));
    if initiated == Some(None) {
        push_problem(&mut problems, SOURCE, id.clone(), "event_date_initiated");
    }
    let mut text = |field, value| {
        let admit = if matches!(field, "res_event_number" | "product_code" | "recall_status") {
            crate::mapped::field_code_text
        } else {
            field_text
        };
        admit(&mut problems, SOURCE, id.clone(), field, value)
    };
    let header = DeviceRecallHeader {
        recall_number: row.product_res_number,
        event_number: text("res_event_number", row.res_event_number),
        firm: text("recalling_firm", row.recalling_firm),
        product_code: text("product_code", row.product_code),
        status: text("recall_status", row.recall_status),
        initiated: initiated.flatten(),
    };
    let reason = text("reason_for_recall", row.reason_for_recall);
    let root_cause = text("root_cause_description", row.root_cause_description);
    let product_description = text("product_description", row.product_description);
    let distribution_pattern = text("distribution_pattern", row.distribution_pattern);
    let k_numbers = list(row.k_numbers, "k_numbers", id.clone(), &mut problems);
    let pma_numbers = list(row.pma_numbers, "pma_numbers", id, &mut problems);
    Mapped {
        refusals: crate::Refusal::None,
        value: DeviceRecallCard {
            header,
            reason,
            root_cause,
            product_description,
            distribution_pattern,
            classification: None,
            k_numbers,
            pma_numbers,
        },
        problems,
    }
}
pub(super) fn classification(row: raw::Enforcement) -> Mapped<Option<Text>> {
    let mut problems = Vec::new();
    let value = crate::mapped::field_code_text(
        &mut problems,
        SOURCE,
        record(row.recall_number),
        "classification",
        row.classification,
    );
    Mapped {
        refusals: crate::Refusal::None,
        value,
        problems,
    }
}
