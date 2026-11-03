//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
fn probe(leg: &biomcp_fetch::Leg<'_>) -> Result<(), biomcp_fetch::RequestError> {
    crate::adverse_event_sources::components::gmdn(leg)?;
    (Probe::Get("", &[("limit", Arg::Num(1))])).run(&super::SPEC, leg)
}
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "FDA MAUDE",
    registry_order: Some(58),
    probe: Probe::Custom(probe),
    components: &[GMDN_COMPONENT],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};

pub(crate) static GMDN: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Gmdn,
        &super::SPEC,
        GMDN_COMPONENT.terms,
    ),
    display: "GMDN",
    registry_order: None,
    probe: Probe::Component,
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};

pub(crate) const GMDN_COMPONENT: crate::registry::Component = crate::registry::Component {
    source: biomcp_types::Source::Maude,
    native_source: Some(biomcp_types::Source::Gmdn),
    name: "gmdn",
    terms: biomcp_fetch::Terms {
        name: "openFDA license: GMDN third-party terminology exceptions",
        url: "https://open.fda.gov/license/",
        usage: biomcp_types::Use::Attribution,
    },
};
