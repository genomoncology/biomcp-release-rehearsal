use crate::openfda::{Envelope, Item};
pub(super) type Response = Envelope<Header>;
use biodata::ReportId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct Header {
    pub safetyreportid: ReportId,
    pub safetyreportversion: Option<crate::raw::Field<Box<str>>>,
    pub serious: Option<crate::raw::Field<Box<str>>>,
    pub receivedate: Option<crate::raw::Field<Box<str>>>,
}

impl Item for Header {}
