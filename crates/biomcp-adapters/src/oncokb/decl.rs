//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "OncoKB",
    registry_order: Some(18),
    probe: Probe::Custom(|leg| leg.probe(&crate::oncokb::INFO, biomcp_fetch::Input::Params(&[]))),
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
    name: biomcp_fetch::SecretName::OncokbApiKey,
    suffix: "",
    word: "oncokb",
    order: 3,
};
