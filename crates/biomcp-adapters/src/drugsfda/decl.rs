//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "Drugs@FDA",
    registry_order: Some(48),
    probe: Probe::Get("", &[("limit", Arg::Num(1))]),
    components: &[],
    auxiliary: &[&super::DOCUMENTS],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};
