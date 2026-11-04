//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "European Medicines Agency",
    registry_order: Some(50),
    probe: Probe::Get("/en/about-us/about-website/legal-notice", &[]),
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
    help: crate::decl::bundle::REPLACE_HELP,
    name: "ema",
    source: biomcp_types::Source::Ema,
    order: 5,
    sync: |fetch| crate::decl::BundleReply::Standard(super::sync(fetch), false),
    state: super::bundle_state,
};
impl crate::decl::Bundle {
    /// Synchronize the European Medicines Agency local catalog.
    pub const EMA: Self = Self(&BUNDLE);
}
