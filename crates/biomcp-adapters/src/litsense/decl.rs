//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "LitSense",
    registry_order: Some(36),
    probe: Probe::Custom(|leg| {
        leg.probe(
            &crate::litsense::SENTENCES,
            biomcp_fetch::Input::Params(&[
                (
                    "query",
                    biomcp_fetch::Arg::Text("AKT1 E17K capivasertib breast cancer"),
                ),
                ("rerank", biomcp_fetch::Arg::Text("true")),
            ]),
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
