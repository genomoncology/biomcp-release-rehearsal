//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "WikiPathways",
    registry_order: Some(25),
    probe: Probe::Get("/help.html", &[]),
    components: &[],
    auxiliary: &[],
    bundles: &[&BUNDLE],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};
pub(crate) static BUNDLE: crate::decl::BundleDecl = crate::decl::BundleDecl {
    help: "Replace WikiPathways search/get metadata catalogs atomically; reads never sync. \
        Membership names a source and release; it implies no therapy, response or actionability. \
        No enrichment is calculated.",
    name: "wikipathways",
    source: biomcp_types::Source::WikiPathways,
    order: 3,
    sync: |fetch| crate::decl::BundleReply::WikiPathways(super::sync(fetch)),
    state: super::bundle_state,
};
impl crate::decl::Bundle {
    /// Synchronize the WikiPathways local catalog.
    pub const WIKI_PATHWAYS: Self = Self(&BUNDLE);
}
