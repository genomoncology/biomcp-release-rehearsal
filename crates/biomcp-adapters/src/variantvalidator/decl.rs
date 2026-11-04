//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "VariantValidator",
    registry_order: Some(14),
    probe: Probe::Get(
        "/variantvalidator/GRCh38/LRG_321p1%3Ap.Arg273His/all",
        &[("content-type", Arg::Text("application/json"))],
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
