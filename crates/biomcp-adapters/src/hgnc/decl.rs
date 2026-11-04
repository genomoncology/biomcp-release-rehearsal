//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "HGNC",
    registry_order: Some(16),
    probe: Probe::Custom(crate::hgnc::probe),
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
    help: crate::decl::bundle::REFRESH_HELP,
    name: "hgnc",
    source: biomcp_types::Source::Hgnc,
    order: 0,
    sync: |fetch| crate::decl::BundleReply::Standard(super::sync_all(fetch), false),
    state: super::bundle_state,
};
impl crate::decl::Bundle {
    /// Synchronize the HGNC local catalog.
    pub const HGNC: Self = Self(&BUNDLE);
}
