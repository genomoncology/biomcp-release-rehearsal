use super::raw;
use biodata::{Attribution, AuthorizationDate, AuthorizationEvent, DrugIdentityLevel, Text};
use biomcp_types::{DrugCard, DrugHeader, DrugSection, DrugSource};
pub(super) fn header(row: &raw::Medicine, problems: &mut crate::mapped::Problems) -> DrugHeader {
    problems.required(
        "_bundle_meta.timestamp",
        row.meta
            .timestamp
            .as_ref()
            .and_then(crate::raw::Field::as_ref),
    );
    DrugHeader {
        matched_name: None,
        id: row.id.clone(),
        source: DrugSource::Ema,
        name: {
            let name = item(row, problems, "name_of_medicine");
            problems.required("name_of_medicine", name)
        },
        version: None,
        // The feed's category describes Human/Veterinary, not identity granularity.
        identity_level: DrugIdentityLevel::Unknown,
        r#type: {
            let category = item(row, problems, "category");
            problems.required("category", category)
        },
        url: item(row, problems, "medicine_url"),
    }
}
pub(super) fn card(row: &raw::Medicine, problems: &mut crate::mapped::Problems) -> DrugCard {
    DrugCard {
        header: header(row, problems),
        alternate_names: Box::default(),
        cross_references: Box::default(),
        parents: Box::default(),
        products: Box::default(),
        formulations: Box::default(),
        version: item(row, problems, "revision_number"),
        updated: problems.optional(
            "last_updated_date",
            row.field("last_updated_date")
                .filter(|value| !matches!(value, Ok("")))
                .map(|value| value.and_then(raw::date)),
        ),
        retrieved: None,
        available_sections: vec![DrugSection::Regulatory].into(),
        next: Box::default(),
    }
}
pub(super) fn event(
    row: &raw::Medicine,
    problems: &mut crate::mapped::Problems,
) -> AuthorizationEvent {
    let dates = row
        .fields
        .iter()
        .filter(|(key, _)| key.ends_with("_date"))
        .filter_map(|(key, _)| {
            Some(AuthorizationDate {
                role: problems.optional(key, Some(Text::new(key)))?,
                date: problems.optional(
                    key,
                    row.field(key)
                        .filter(|value| !matches!(value, Ok("")))
                        .map(|value| value.and_then(raw::date)),
                )?,
            })
        })
        .collect();
    AuthorizationEvent {
        record: row.id.clone(),
        product_number: None,
        jurisdiction: None,
        status: {
            let status = item(row, problems, "medicine_status");
            problems.required("medicine_status", status)
        },
        kind: item(row, problems, "opinion_status"),
        dates,
        indication: item(row, problems, "therapeutic_indication"),
        attribution: Attribution {
            approved: None,
            published: None,
            source: row.source.clone(),
            record: Some(row.record.clone()),
            version: item(row, problems, "revision_number"),
            release: problems.required(
                "_bundle_meta.timestamp",
                row.meta
                    .timestamp
                    .as_ref()
                    .and_then(crate::raw::Field::as_ref)
                    .cloned(),
            ),
            submitter: item(
                row,
                problems,
                "marketing_authorisation_developer_applicant_holder",
            ),
            evaluated: None,
            url: item(row, problems, "medicine_url"),
        },
    }
}

fn item(row: &raw::Medicine, problems: &mut crate::mapped::Problems, field: &str) -> Option<Text> {
    problems.optional(
        field,
        row.field(field)
            .map(|value| value.and_then(crate::mapped::source_text)),
    )
}
