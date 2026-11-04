//! Organization rows carry published details without inventing an organization identity.
use biodata::Text;
use serde::{Deserialize, Serialize};
/// One NCI organization search row. Names are not unique; counts cover trials of every status.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Organization {
    /// Exact publisher organization name, never a unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<Text>,
    /// Search trials at this exact organization name; names may identify several locations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<crate::Step>,
    /// NCI trial count across all statuses, including closed trials.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    /// Native city.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_city: Option<Text>,
    /// Native state or province.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_state_or_province: Option<Text>,
    /// Native country.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_country: Option<Text>,
    /// First native address line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_address_line_1: Option<Text>,
    /// Second native address line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_address_line_2: Option<Text>,
    /// Native postal code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_postal_code: Option<Text>,
    /// Native organization family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_family: Option<Text>,
    /// Published organization email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_email: Option<Text>,
    /// Published organization fax.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_fax: Option<Text>,
    /// Published public contact name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_name: Option<Text>,
    /// Published public contact telephone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_phone: Option<Text>,
    /// Published public contact email.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<Text>,
    /// Native geographic coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_coordinates: Option<OrganizationCoordinates>,
    /// Native Veterans Affairs marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub org_va: Option<bool>,
}
/// Geographic point published by NCI, in degrees.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrganizationCoordinates {
    /// Latitude in degrees.
    pub lat: f64,
    /// Longitude in degrees.
    pub lon: f64,
}
/// Local page of a complete bounded source observation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct OrganizationPage {
    /// Published organizations in source order, with optional source details.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[Organization]>,
}
impl crate::ResponseBody for OrganizationPage {
    const SCHEMA: &'static str = "biomcp.organization-page/4";
    type Query = crate::OrganizationSearch;
}
