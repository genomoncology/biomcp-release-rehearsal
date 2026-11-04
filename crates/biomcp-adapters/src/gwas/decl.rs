//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "GWAS Catalog",
    registry_order: Some(10),
    probe: Probe::Get(
        "/associations",
        &[("rs_id", Arg::Text("rs113488022")), ("size", Arg::Num(1))],
    ),
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: Some(
            "BioMCP's local request deadline expired. GWAS Catalog response times can vary; retry later or use \
            https://www.ebi.ac.uk/gwas/. Contact the GWAS Catalog helpdesk if this persists.",
        ),
        quota: None,
    },
    key: None,
};
