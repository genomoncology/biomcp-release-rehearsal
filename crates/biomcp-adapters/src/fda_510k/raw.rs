use crate::openfda::{Envelope, Item};
pub(super) type Response = Envelope<Record>;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Record {
    pub k_number: biodata::DiagnosticDeviceId,
    pub device_name: Option<crate::raw::Field<Box<str>>>,
    pub applicant: Option<crate::raw::Field<Box<str>>>,
    pub decision_code: Option<crate::raw::Field<Box<str>>>,
    pub decision_description: Option<crate::raw::Field<Box<str>>>,
    pub decision_date: Option<crate::raw::Field<Box<str>>>,
    pub date_received: Option<crate::raw::Field<Box<str>>>,
    pub product_code: Option<crate::raw::Field<Box<str>>>,
}
impl Record {
    pub(super) fn mapped(self) -> crate::Mapped<biomcp_types::DiagnosticClearance> {
        let mut p = crate::mapped::Problems {
            source: biomcp_types::Source::Fda510k,
            record: crate::mapped::record(&self.k_number),
            list: Vec::new(),
        };
        let value = biomcp_types::DiagnosticClearance {
            k_number: self.k_number,
            device_name: p.field("device_name", self.device_name),
            applicant: p.field("applicant", self.applicant),
            decision_code: p.field("decision_code", self.decision_code),
            decision_description: p.field("decision_description", self.decision_description),
            decision_date: p.field("decision_date", self.decision_date),
            date_received: p.field("date_received", self.date_received),
            product_code: p.field("product_code", self.product_code),
        };
        crate::Mapped {
            refusals: crate::Refusal::None,
            value,
            problems: p.list,
        }
    }
}

impl Item for Record {}
