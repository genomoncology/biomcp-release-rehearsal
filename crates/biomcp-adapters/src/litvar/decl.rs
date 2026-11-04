//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "LitVar",
    registry_order: Some(35),
    probe: Probe::Custom(|leg| {
        leg.probe(
            &crate::litvar::AUTOCOMPLETE,
            biomcp_fetch::Input::Params(&[("query", biomcp_fetch::Arg::Text("rs121434592"))]),
        )
    }),
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};
