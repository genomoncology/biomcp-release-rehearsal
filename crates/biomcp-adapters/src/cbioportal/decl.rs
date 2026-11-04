//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
use biomcp_fetch::Arg;
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "cBioPortal",
    registry_order: Some(43),
    probe: Probe::Get("/studies", &[("pageSize", Arg::Num(1))]),
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
    name: "cbioportal-studies",
    source: biomcp_types::Source::Cbioportal,
    order: 4,
    sync: |fetch| match super::sync_studies(fetch) {
        Ok((mapped, stale)) => crate::decl::BundleReply::Standard(Ok(mapped), stale),
        Err(error) => crate::decl::BundleReply::Standard(Err(error), false),
    },
    state: super::bundle_state,
};
impl crate::decl::Bundle {
    /// Synchronize the cBioPortal local catalog.
    pub const CBIOPORTAL_STUDIES: Self = Self(&BUNDLE);
}
