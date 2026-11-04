use crate::openfda::Envelope;
pub(super) type Response<T> = Envelope<T>;
use crate::raw::Field;
use biodata::{DeviceRecallNumber, PartialDate, Text};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Record {
    pub product_res_number: DeviceRecallNumber,
    pub res_event_number: Option<Field<String>>,
    pub recalling_firm: Option<Field<String>>,
    pub product_code: Option<Field<String>>,
    pub recall_status: Option<Field<String>>,
    pub event_date_initiated: Option<Field<PartialDate>>,
    pub reason_for_recall: Option<Field<String>>,
    pub root_cause_description: Option<Field<String>>,
    pub product_description: Option<Field<String>>,
    pub distribution_pattern: Option<Field<String>>,
    pub k_numbers: Option<Field<Vec<Field<Text>>>>,
    pub pma_numbers: Option<Field<Vec<Field<Text>>>>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Enforcement {
    pub recall_number: DeviceRecallNumber,
    pub classification: Option<Field<String>>,
}
pub(super) trait Identity {
    fn id(&self) -> &DeviceRecallNumber;
}
impl Identity for Record {
    fn id(&self) -> &DeviceRecallNumber {
        &self.product_res_number
    }
}
impl Identity for Enforcement {
    fn id(&self) -> &DeviceRecallNumber {
        &self.recall_number
    }
}

impl crate::openfda::Item for Record {}

impl crate::openfda::Item for Enforcement {}
