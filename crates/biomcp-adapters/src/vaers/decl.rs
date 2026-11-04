//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "CDC WONDER VAERS",
    registry_order: Some(59),
    probe: Probe::Get(crate::vaers::HEALTH_PATH, &[]),
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: Some(
            "CDC WONDER can take longer than BioMCP's local command deadline. Use the interactive form at \
            https://wonder.cdc.gov/vaers.html for a longer query. See https://wonder.cdc.gov/wonder/help/faq.html.",
        ),
        quota: Some(WONDER_QUOTA),
    },
    key: None,
};

/// Explain immediate local refusal under CDC's automated-query spacing guidance.
pub const WONDER_QUOTA: &str = "BioMCP's local quota refused this request before contacting CDC WONDER. Retry after the reported seconds \
            left. Automated queries must follow the spacing at https://wonder.cdc.gov/wonder/help/wonder-api.html.";
