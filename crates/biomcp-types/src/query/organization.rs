//! Checked NCI organization selectors preserve native matching.
use crate::Paged;
use crate::QueryInvalid;
use biodata::Text;
use serde::{Deserialize, Serialize};
biodata::wire_enum! {
    /// Organization discovery source.
    OrganizationSource { /// NCI Clinical Trials Search.
        Nci => "nci" }
}
/// Find NCI organizations. Names are not unique; trial counts cover every status, including closed trials. No organization ID or get is available.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields")]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(with = "Fields"))]
pub struct OrganizationSearch(Fields);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
struct Fields {
    /// Explicit keyed NCI source.
    source: OrganizationSource,
    /// NCI partial organization name; names are not unique.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    name: Option<Text>,
    /// Native organization city.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    city: Option<Text>,
    /// Native state or province.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    state: Option<Text>,
    /// Native organization country.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    country: Option<Text>,
    /// Native postal code.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    postal_code: Option<Text>,
    /// Native organization family.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    family: Option<Text>,
    /// Radius center latitude in degrees; requires lon and distance.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    lat: Option<Text>,
    /// Radius center longitude in degrees; requires lat and distance.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    lon: Option<Text>,
    /// Positive radius with explicit mi or km units.
    #[serde(
        default,
        deserialize_with = "super::gene::present",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    distance: Option<Text>,
    /// Five headers by default, at most fifty.
    #[serde(default = "limit")]
    #[cfg_attr(feature = "schema", schemars(range(min = 1, max = 50)))]
    limit: u32,
    /// Offset within the complete retained organization snapshot.
    #[serde(default)]
    offset: u32,
    /// Snapshot identity retained by continuation commands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pool: Option<Text>,
}
fn limit() -> u32 {
    5
}
crate::query::validation::checked_query!(OrganizationSearch, Fields);
impl TryFrom<Fields> for OrganizationSearch {
    type Error = QueryInvalid;
    fn try_from(v: Fields) -> Result<Self, Self::Error> {
        if !(1..=50).contains(&v.limit) {
            return Err(QueryInvalid::new(&["limit"]));
        }
        validate_radius(&v)?;
        Ok(Self(v))
    }
}
fn validate_radius(v: &Fields) -> Result<(), QueryInvalid> {
    match (&v.lat, &v.lon, &v.distance) {
        (None, None, None) => Ok(()),
        (Some(lat), Some(lon), Some(distance)) => {
            let lat = lat
                .as_str()
                .parse::<f64>()
                .ok()
                .filter(|n| (-90.0..=90.0).contains(n));
            let lon = lon
                .as_str()
                .parse::<f64>()
                .ok()
                .filter(|n| (-180.0..=180.0).contains(n));
            let distance = distance
                .as_str()
                .strip_suffix("mi")
                .or_else(|| distance.as_str().strip_suffix("km"))
                .and_then(|n| n.parse::<f64>().ok())
                .filter(|n| n.is_finite() && *n > 0.0);
            if lat.is_none() || lon.is_none() || distance.is_none() {
                return Err(QueryInvalid::new(&["lat", "lon", "distance"]));
            }
            Ok(())
        }
        _ => Err(QueryInvalid::new(&["lat", "lon", "distance"])),
    }
}
impl OrganizationSearch {
    /// NCI partial organization name; names are not unique.
    pub fn name(&self) -> Option<&Text> {
        self.0.name.as_ref()
    }
    /// Native organization city.
    pub fn city(&self) -> Option<&Text> {
        self.0.city.as_ref()
    }
    /// Native state or province.
    pub fn state(&self) -> Option<&Text> {
        self.0.state.as_ref()
    }
    /// Native organization country.
    pub fn country(&self) -> Option<&Text> {
        self.0.country.as_ref()
    }
    /// Native postal code.
    pub fn postal_code(&self) -> Option<&Text> {
        self.0.postal_code.as_ref()
    }
    /// Native organization family.
    pub fn family(&self) -> Option<&Text> {
        self.0.family.as_ref()
    }
    /// Radius center latitude in degrees; requires lon and distance.
    pub fn lat(&self) -> Option<&Text> {
        self.0.lat.as_ref()
    }
    /// Radius center longitude in degrees; requires lat and distance.
    pub fn lon(&self) -> Option<&Text> {
        self.0.lon.as_ref()
    }
    /// Positive radius with explicit mi or km units.
    pub fn distance(&self) -> Option<&Text> {
        self.0.distance.as_ref()
    }
    /// Requested local page size.
    pub fn limit(&self) -> u32 {
        self.0.limit
    }
    /// Local snapshot position.
    pub fn offset(&self) -> u32 {
        self.0.offset
    }
    /// Retained observation identity.
    pub fn pool(&self) -> Option<&Text> {
        self.0.pool.as_ref()
    }
    /// Continue the same observation at a local offset.
    pub fn at_page(&self, offset: u32, pool: Text) -> Self {
        let mut q = self.clone();
        q.0.offset = offset;
        q.0.pool = Some(pool);
        q
    }

    /// Snapshot key selectors exclude page controls.
    pub fn snapshot_query(&self) -> Self {
        let mut q = self.restart();
        q.0.limit = limit();
        q
    }
}
impl crate::Paged for OrganizationSearch {
    type Offset = ();
    fn set_page_offset(&mut self, _offset: Self::Offset) {
        self.0.offset = 0;
    }
    fn clear_page_context(&mut self) {
        self.0.pool = None;
    }
}
