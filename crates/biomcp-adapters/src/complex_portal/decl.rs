//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "Complex Portal",
    registry_order: Some(23),
    probe: Probe::Get(
        "/search/P15056",
        &[
            ("first", Arg::Num(0)),
            ("number", Arg::Num(5)),
            ("filters", Arg::Text("species_f:(\"Homo sapiens\")")),
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
