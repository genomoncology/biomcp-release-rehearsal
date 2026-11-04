//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "AlphaGenome",
    registry_order: Some(19),
    probe: Probe::Unavailable,
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: Some(&KEY),
};

pub(crate) static KEY: crate::decl::KeyDecl = crate::decl::KeyDecl {
    spec: &super::SPEC,
    name: biomcp_fetch::SecretName::AlphaGenomeApiKey,
    suffix: "",
    word: "alphagenome",
    order: 4,
};
