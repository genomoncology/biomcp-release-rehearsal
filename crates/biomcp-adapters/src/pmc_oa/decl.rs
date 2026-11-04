//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "PMC Open Access",
    registry_order: Some(38),
    probe: Probe::Get(
        "/",
        &[
            ("list-type", Arg::Num(2)),
            ("prefix", Arg::Text("PMC5606870.")),
            ("max-keys", Arg::Num(1)),
        ],
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
