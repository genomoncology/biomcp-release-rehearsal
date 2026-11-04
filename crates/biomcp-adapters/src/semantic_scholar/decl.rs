//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "Semantic Scholar",
    registry_order: Some(40),
    probe: Probe::Get(
        "/paper/search",
        &[
            ("query", Arg::Text("EGFR")),
            ("limit", Arg::Num(1)),
            ("fields", Arg::Text("externalIds,title,year")),
        ],
    ),
    components: &[],
    auxiliary: &[
        &crate::semantic_scholar::recommend::RECOMMEND_SPEC,
        &crate::semantic_scholar::pdf::PUBLISHER,
    ],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: Some(&KEY),
};

pub(crate) static KEY: crate::decl::KeyDecl = crate::decl::KeyDecl {
    spec: &super::SPEC,
    name: biomcp_fetch::SecretName::SemanticScholarApiKey,
    suffix: "; a free key raises the request limit",
    word: "semantic-scholar",
    order: 1,
};
