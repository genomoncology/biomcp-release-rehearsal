//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "InterPro",
    registry_order: Some(22),
    probe: Probe::Get(
        "/entry/interpro/protein/uniprot/P00533/",
        &[("page_size", Arg::Num(5))],
    ),
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: Some(
            "BioMCP's local request deadline expired. The health probe uses a shorter local cap than protein requests. \
            InterPro documents that some queries take minutes; this timeout does not establish an outage. Please \
            retry later or check https://www.ebi.ac.uk/interpro/. See https://interpro-documentation.readthedocs.io/en/latest/faq.html.",
        ),
        quota: None,
    },
    key: None,
};
