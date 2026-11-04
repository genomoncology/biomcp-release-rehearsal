//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "ClinGen Evidence Repository",
    registry_order: Some(29),
    probe: Probe::Get(
        "/evrepo/api/summary/classifications",
        &[
            ("columns", Arg::Text("caId")),
            ("values", Arg::Text("CA000434")),
            ("matchTypes", Arg::Text("exact")),
            ("pgSize", Arg::Num(1)),
            ("pg", Arg::Num(1)),
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
