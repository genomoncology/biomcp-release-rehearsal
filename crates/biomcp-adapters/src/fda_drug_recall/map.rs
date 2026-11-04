use super::raw;
use crate::{
    mapped::{Mapped, field_text, push_problem, record},
    raw::Field,
};
use biodata::{PartialDate, Text};
use biomcp_types::{DrugRecallCard, DrugRecallHeader, Problem, Source};
const SOURCE: Source = Source::FdaDrugRecall;
fn date(
    value: Option<Field<String>>,
    field: &str,
    id: Option<Text>,
    problems: &mut Vec<Problem>,
) -> Option<PartialDate> {
    let value = value?;
    let date = value.value().and_then(|v| {
        if v.len() != 8 || !v.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let year = v.get(..4)?.parse().ok()?;
        let month = v.get(4..6)?.parse().ok()?;
        let day = v.get(6..)?.parse().ok()?;
        PartialDate::new(year, Some(month), Some(day)).ok()
    });
    if date.is_none() {
        push_problem(problems, SOURCE, id, field);
    }
    date
}
pub(super) fn mapped(row: raw::Record) -> Mapped<DrugRecallCard> {
    let mut problems = Vec::new();
    let id = record(&row.recall_number);
    let initiated = date(
        row.recall_initiation_date,
        "recall_initiation_date",
        id.clone(),
        &mut problems,
    );
    let reported = date(row.report_date, "report_date", id.clone(), &mut problems);
    let mut text = |field, value| {
        let admit = if matches!(
            field,
            "classification" | "status" | "code_info" | "voluntary_mandated"
        ) {
            crate::mapped::field_code_text
        } else {
            field_text
        };
        admit(&mut problems, SOURCE, id.clone(), field, value)
    };
    let header = DrugRecallHeader {
        recall_number: row.recall_number,
        classification: text("classification", row.classification),
        status: text("status", row.status),
        firm: text("recalling_firm", row.recalling_firm),
        product_description: text("product_description", row.product_description),
        initiated,
        reported,
    };
    let value = DrugRecallCard {
        header,
        reason: text("reason_for_recall", row.reason_for_recall),
        distribution_pattern: text("distribution_pattern", row.distribution_pattern),
        code_info: text("code_info", row.code_info),
        voluntary_mandated: text("voluntary_mandated", row.voluntary_mandated),
    };
    Mapped {
        refusals: crate::Refusal::None,
        value,
        problems,
    }
}
