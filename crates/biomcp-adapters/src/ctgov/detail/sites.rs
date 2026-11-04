//! Provider contacts and sites mapped once into source-preserving values.
use biodata::{TrialContact, TrialCoordinates, TrialLocation, TrialOfficial};

use crate::{mapped::Problems, raw::Field};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ContactsLocations {
    central_contacts: Option<Field<Vec<Field<Contact>>>>,
    overall_officials: Option<Field<Vec<Field<Official>>>>,
    locations: Option<Field<Vec<Field<Location>>>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Contact {
    name: Option<Field<Box<str>>>,
    role: Option<Field<biodata::TrialContactRole>>,
    phone: Option<Field<Box<str>>>,
    phone_ext: Option<Field<Box<str>>>,
    email: Option<Field<Box<str>>>,
}
#[derive(Deserialize)]
struct Official {
    name: Option<Field<Box<str>>>,
    affiliation: Option<Field<Box<str>>>,
    role: Option<Field<biodata::TrialOfficialRole>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Location {
    facility: Option<Field<Box<str>>>,
    city: Option<Field<Box<str>>>,
    state: Option<Field<Box<str>>>,
    zip: Option<Field<Box<str>>>,
    country: Option<Field<Box<str>>>,
    status: Option<Field<biodata::TrialSiteStatus>>,
    // Coordinate aliases belong to this provider's shape, not the shared domain value.
    geo_point: Option<Field<Point>>,
    contacts: Option<Field<Vec<Field<Contact>>>>,
}
type SiteRows = (
    Box<[TrialLocation]>,
    Box<[TrialContact]>,
    Box<[TrialOfficial]>,
);
impl ContactsLocations {
    pub(super) fn rows(self, problems: &mut Problems, refusals: &mut crate::Refusal) -> SiteRows {
        let locations =
            super::super::refusals::records(problems, refusals, "locations", self.locations)
                .into_vec()
                .into_iter()
                .map(|v| v.mapped(problems, refusals))
                .collect();
        let contacts = super::super::refusals::records(
            problems,
            refusals,
            "centralContacts",
            self.central_contacts,
        )
        .into_vec()
        .into_iter()
        .map(|v| v.mapped(problems))
        .collect();
        let officials = super::super::refusals::records(
            problems,
            refusals,
            "overallOfficials",
            self.overall_officials,
        )
        .into_vec()
        .into_iter()
        .map(|v| TrialOfficial {
            name: problems.native_text("overallOfficials.name", v.name),
            affiliation: problems.native_text("overallOfficials.affiliation", v.affiliation),
            role: problems.field("overallOfficials.role", v.role),
        })
        .collect();
        (locations, contacts, officials)
    }
}

impl Contact {
    fn mapped(self, problems: &mut Problems) -> TrialContact {
        TrialContact {
            name: problems.native_text("name", self.name),
            role: problems.field("role", self.role),
            phone: problems.checked_native_text("phone", self.phone),
            phone_ext: problems.checked_native_text("phoneExt", self.phone_ext),
            email: problems.checked_native_text("email", self.email),
        }
    }
}
impl Location {
    fn mapped(self, problems: &mut Problems, refusals: &mut crate::Refusal) -> TrialLocation {
        TrialLocation {
            facility: problems.native_text("facility", self.facility),
            city: problems.native_text("city", self.city),
            state: problems.native_text("state", self.state),
            zip: problems.checked_native_text("zip", self.zip),
            country: problems.native_text("country", self.country),
            status: problems.field("locations.status", self.status),
            coordinates: problems.field("geoPoint", self.geo_point).map(|v| v.0),
            contacts: super::super::refusals::records(
                problems,
                refusals,
                "locations.contacts",
                self.contacts,
            )
            .into_vec()
            .into_iter()
            .map(|v| v.mapped(problems))
            .collect(),
        }
    }
}

#[derive(Deserialize)]
#[serde(try_from = "RawPoint")]
struct Point(TrialCoordinates);
#[derive(Deserialize)]
struct RawPoint {
    lat: biodata::FiniteNumber,
    lon: biodata::FiniteNumber,
}
impl TryFrom<RawPoint> for Point {
    type Error = biodata::Error;
    fn try_from(v: RawPoint) -> Result<Self, Self::Error> {
        TrialCoordinates::new(v.lat, v.lon).map(Self)
    }
}
