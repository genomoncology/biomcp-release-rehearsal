//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "ClinPGx",
    registry_order: Some(2),
    probe: Probe::Get(
        crate::clinpgx::CLINICAL_PATH,
        &[
            ("location.genes.symbol", Arg::Text("DPYD")),
            ("view", Arg::Text("min")),
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
