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
        Self {
            id: row.id,
            set_id: row.set_id,
            version: p.field("version", row.version),
            effective_time,
            openfda,
            spl_product_data_elements: p
                .optional_fields("spl_product_data_elements", row.spl_product_data_elements),
            boxed_warning: p.optional_fields("boxed_warning", row.boxed_warning),
            dosage_and_administration: p
                .optional_fields("dosage_and_administration", row.dosage_and_administration),
            drug_interactions: p.optional_fields("drug_interactions", row.drug_interactions),
            indications_and_usage: p
                .optional_fields("indications_and_usage", row.indications_and_usage),
            warnings_and_cautions: p
                .optional_fields("warnings_and_cautions", row.warnings_and_cautions),
            problems: p.list,
        }
    }
}
