//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "WHO prequalified medicine",
    registry_order: Some(5),
    probe: Probe::Get(crate::who_pq::HEALTH_PATH, &[]),
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
    name: "who-pq",
    source: biomcp_types::Source::WhoPq,
    order: 7,
    sync: |fetch| crate::decl::BundleReply::Standard(super::sync(fetch), false),
    state: super::bundle_state,
};
impl crate::decl::Bundle {
    /// Synchronize the WHO prequalified medicine local catalog.
    pub const WHO_PQ: Self = Self(&BUNDLE);
}
