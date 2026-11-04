//! Reviewed field ownership; no dictionaries, inferred mappings or fetch-and-discard.
use crate::registry::Component;
use biomcp_fetch::{Leg, RequestError, SourceError};
use biomcp_types::{Reason, Source};
/// Exact native field paths owned by the MedDRA component.
pub const MEDDRA_FIELDS: &[&str] = &[
    "patient.reaction.reactionmeddrapt",
    "patient.reaction.reactionmeddraversionpt",
];
/// Exact native field paths conservatively owned by GMDN; no other fields are assigned.
/// This assignment does not establish that every native string is a GMDN term.
pub const GMDN_FIELDS: &[&str] = &["device.generic_name"];
/// Admit a FAERS reaction section or reaction-filtered search before endpoint/cache/replay access.
/// Ordinary search, card and drugs reads do not require this terminology component.
pub fn meddra(leg: &Leg<'_>) -> Result<(), RequestError> {
    admit(leg, Source::Meddra, Source::Faers)
}
/// Admit a MAUDE positional search or devices section before endpoint/cache/replay access.
/// Card and narrative reads do not require this terminology component.
pub fn gmdn(leg: &Leg<'_>) -> Result<(), RequestError> {
    admit(leg, Source::Gmdn, Source::Maude)
}
fn admit(leg: &Leg<'_>, component: Source, owner: Source) -> Result<(), RequestError> {
    if all()
        .find(|c| c.native_source == Some(component))
        .is_none_or(|c| c.disabled_on_leg(leg))
    {
        Err(SourceError {
            source: owner,
            reason: Reason::Disabled,
            status: None,
            tls_cause: None,
            retry_after: None,
        }
        .into())
    } else {
        Ok(())
    }
}
/// Label one carried terminology using the same canonical declaration as admission.
pub fn labels(
    owner: Source,
    component: Source,
) -> Result<Box<[biomcp_types::ComponentLabel]>, RequestError> {
    let declaration = all()
        .find(|v| v.source == owner && v.native_source == Some(component))
        .ok_or(biomcp_fetch::InputError::Serialization)?;
    let text = |value: &str| {
        biodata::Text::new(value).map_err(|_| biomcp_fetch::InputError::Serialization)
    };
    Ok(vec![biomcp_types::ComponentLabel {
        component: text(declaration.name)?,
        terms: biomcp_types::ComponentTerms {
            usage: declaration.terms.usage,
            name: text(declaration.terms.name)?,
            url: text(declaration.terms.url)?,
        },
    }]
    .into())
}

/// Enumerate independently labeled fields from their adapter declarations.
pub fn all() -> impl Iterator<Item = &'static Component> {
    crate::registry::components()
        .filter(|component| matches!(component.source, Source::Faers | Source::Maude))
}
