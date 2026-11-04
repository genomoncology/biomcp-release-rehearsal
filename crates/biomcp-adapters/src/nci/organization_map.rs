//! Map published organization details with rejected fields disclosed once.
use super::organization_raw::Page;
use crate::{Mapped, mapped::Problems};
use biomcp_types::{Organization, Source};
pub(super) fn page(raw: Page) -> Mapped<Vec<Organization>> {
    let mut problems = Vec::new();
    let value = raw
        .data
        .into_iter()
        .map(|row| organization(row, &mut problems))
        .collect();
    Mapped {
        refusals: crate::Refusal::None,
        value,
        problems,
    }
}

fn organization(
    row: super::organization_raw::Organization,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Organization {
    let mut p = Problems {
        source: Source::Nci,
        record: None,
        list: Vec::new(),
    };
    let value = Organization {
        next: None,
        name: p.source_field("name", row.name),
        count: p.field("count", row.count),
        org_city: p.optional(
            "org_city",
            row.org_city.map(|v| v.value().map(Into::into).ok_or(())),
        ),
        org_state_or_province: p.optional(
            "org_state_or_province",
            row.org_state_or_province
                .map(|v| v.value().map(Into::into).ok_or(())),
        ),
        org_country: p.optional(
            "org_country",
            row.org_country.map(|v| v.value().map(Into::into).ok_or(())),
        ),
        org_address_line_1: p.optional(
            "org_address_line_1",
            row.org_address_line_1
                .map(|v| v.value().map(Into::into).ok_or(())),
        ),
        org_address_line_2: p.optional(
            "org_address_line_2",
            row.org_address_line_2
                .map(|v| v.value().map(Into::into).ok_or(())),
        ),
        org_postal_code: p.optional(
            "org_postal_code",
            row.org_postal_code.map(|v| v.value().ok_or(())),
        ),
        org_family: p.optional(
            "org_family",
            row.org_family.map(|v| v.value().map(Into::into).ok_or(())),
        ),
        org_email: p.optional("org_email", row.org_email.map(|v| v.value().ok_or(()))),
        org_fax: p.optional("org_fax", row.org_fax.map(|v| v.value().ok_or(()))),
        contact_name: p.optional(
            "contact_name",
            row.contact_name
                .map(|v| v.value().map(Into::into).ok_or(())),
        ),
        contact_phone: p.optional(
            "contact_phone",
            row.contact_phone.map(|v| v.value().ok_or(())),
        ),
        contact_email: p.optional(
            "contact_email",
            row.contact_email.map(|v| v.value().ok_or(())),
        ),
        org_coordinates: p.optional(
            "org_coordinates",
            row.org_coordinates.map(|v| {
                v.value()
                    .map(|v| v.0)
                    .filter(|v| {
                        (-90.0..=90.0).contains(&v.lat) && (-180.0..=180.0).contains(&v.lon)
                    })
                    .ok_or(())
            }),
        ),
        org_va: p.optional("org_va", row.org_va.map(|v| v.value().ok_or(()))),
    };
    problems.extend(p.list);
    value
}
