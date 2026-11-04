//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "ClinVar",
    registry_order: Some(28),
    probe: Probe::Get(
        "/einfo.fcgi",
        &[("db", Arg::Text("clinvar")), ("retmode", Arg::Text("json"))],
    ),
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};
