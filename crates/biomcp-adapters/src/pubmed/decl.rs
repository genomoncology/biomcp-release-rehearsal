//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "PubMed",
    registry_order: Some(33),
    probe: Probe::Get(
        "/einfo.fcgi",
        &[("db", Arg::Text("pubmed")), ("retmode", Arg::Text("json"))],
    ),
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
    name: biomcp_fetch::SecretName::NcbiApiKey,
    suffix: "",
    word: "ncbi",
    order: 0,
};
