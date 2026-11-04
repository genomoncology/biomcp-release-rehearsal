//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "MyVariant.info",
    registry_order: Some(11),
    probe: Probe::Get("/metadata", &[]),
    components: crate::myvariant::components::COMPONENTS,
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};
