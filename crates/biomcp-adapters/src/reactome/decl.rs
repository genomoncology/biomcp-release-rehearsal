//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "Reactome",
    discovery_aliases: &[],
    registry_order: Some(24),
    probe: Probe::Get("/data/database/version", &[]),
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};
