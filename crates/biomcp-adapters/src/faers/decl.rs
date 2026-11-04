//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
fn probe(leg: &biomcp_fetch::Leg<'_>) -> Result<(), biomcp_fetch::RequestError> {
    crate::adverse_event_sources::components::meddra(leg)?;
    (Probe::Get("", &[("limit", Arg::Num(1))])).run(&super::SPEC, leg)
}
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "FDA FAERS",
    registry_order: Some(57),
    probe: Probe::Custom(probe),
    components: &[MEDDRA_COMPONENT],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};

pub(crate) static MEDDRA: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Meddra,
        &super::SPEC,
        MEDDRA_COMPONENT.terms,
    ),
    display: "MedDRA",
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

pub(crate) const MEDDRA_COMPONENT: crate::registry::Component = crate::registry::Component {
    source: biomcp_types::Source::Faers,
    native_source: Some(biomcp_types::Source::Meddra),
    name: "meddra",
    terms: biomcp_fetch::Terms {
        name: "MedDRA statement on data sharing; third-party terminology",
        url: "https://admin.meddra.org/sites/default/files/page/documents/000198_statement_on_meddra_data_sharing_1_1.pdf",
        usage: biomcp_types::Use::Attribution,
    },
};
