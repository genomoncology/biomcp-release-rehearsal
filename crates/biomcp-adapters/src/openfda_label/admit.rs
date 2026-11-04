//! Keep each valid field of the source label revision.
use super::{
    raw::{Names, Record},
    wire,
};
impl From<wire::Record> for Record {
    fn from(row: wire::Record) -> Self {
        let mut p = crate::mapped::Problems {
            source: biomcp_types::Source::OpenfdaLabel,
            record: crate::mapped::record(&row.id),
            list: Vec::new(),
        };
        let openfda = p
            .field("openfda", row.openfda)
            .map(|v| Names {
                brand_name: p.optional_fields("openfda.brand_name", v.brand_name),
                generic_name: p.optional_fields("openfda.generic_name", v.generic_name),
                spl_set_id: p.optional_fields("openfda.spl_set_id", v.spl_set_id),
                application_number: p
                    .optional_fields("openfda.application_number", v.application_number),
            })
            .unwrap_or(Names {
                brand_name: None,
                generic_name: None,
                spl_set_id: None,
                application_number: None,
            });
        let effective_time = p
            .field("effective_time", row.effective_time)
            .and_then(|v| p.optional("effective_time", Some(crate::drugsfda::date(&v))));
        let sections = row
            .sections
            .into_iter()
            .filter_map(|(heading, value)| {
                if !super::HEADINGS.contains(&heading.as_str()) {
                    return None;
                }
                p.optional_fields(&heading, Some(value))
                    .map(|parts| (heading, parts))
            })
            .collect();
        let mut record = Self {
            id: row.id,
            set_id: row.set_id,
            version: p.field("version", row.version),
            effective_time,
            openfda,
            sections,
            problems: p.list,
        };
        let (_, problems) = record.product_name();
        record.problems.extend(problems);
        record
    }
}
