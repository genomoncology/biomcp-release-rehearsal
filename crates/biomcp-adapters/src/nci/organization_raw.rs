//! Complete native organization observations reject missing totals or cut collections.
use crate::native_text::NativeText;
use crate::raw::Field;
use biodata::Text;
use biomcp_fetch::{BadReply, Reply};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub(super) struct Page {
    pub total: u64,
    error: Option<serde::de::IgnoredAny>,
    pub data: Vec<Organization>,
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.error.is_some() || self.total != self.data.len() as u64 {
            return Err(BadReply);
        }
        Ok(())
    }
}
#[derive(Debug, Deserialize)]
pub(super) struct Organization {
    pub name: Option<Field<NativeText>>,
    pub count: Option<Field<u64>>,
    pub org_city: Option<Field<NativeText>>,
    pub org_state_or_province: Option<Field<NativeText>>,
    pub org_country: Option<Field<NativeText>>,
    pub org_address_line_1: Option<Field<NativeText>>,
    pub org_address_line_2: Option<Field<NativeText>>,
    pub org_postal_code: Option<Field<Text>>,
    pub org_family: Option<Field<NativeText>>,
    pub org_email: Option<Field<Text>>,
    pub org_fax: Option<Field<Text>>,
    pub contact_name: Option<Field<NativeText>>,
    pub contact_phone: Option<Field<Text>>,
    pub contact_email: Option<Field<Text>>,
    pub org_coordinates:
        Option<Field<crate::strict_input::Input<biomcp_types::OrganizationCoordinates>>>,
    pub org_va: Option<Field<bool>>,
}
