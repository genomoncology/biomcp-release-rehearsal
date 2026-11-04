//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "Europe PMC",
    registry_order: Some(34),
    probe: Probe::Get(
        "/search",
        &[
            ("query", Arg::Text("EXT_ID:17611497 AND SRC:MED")),
            ("format", Arg::Text("json")),
            ("pageSize", Arg::Num(1)),
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
