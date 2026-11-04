//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "NIH iCite",
    registry_order: Some(39),
    probe: Probe::Get(
        "/pubs",
        &[("pmids", Arg::Text("17611497")), ("fl", Arg::Text("pmid"))],
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
