//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "MyDisease.info",
    registry_order: Some(53),
    probe: Probe::Get("/metadata", &[]),
    components: &[
        MONDO_COMPONENT,
        DOID_COMPONENT,
        HPO_COMPONENT,
        ORPHANET_COMPONENT,
        OMIM_COMPONENT,
    ],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};

pub(crate) static ORPHANET: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Orphanet,
        &super::SPEC,
        ORPHANET_COMPONENT.terms,
    ),
    display: "Orphadata Science",
    registry_order: None,
    probe: Probe::Component,
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};

pub(crate) static OMIM: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Omim,
        &super::SPEC,
        OMIM_COMPONENT.terms,
    ),
    display: "OMIM",
    registry_order: None,
    probe: Probe::Component,
    components: &[],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};

pub(crate) const MONDO_COMPONENT: crate::registry::Component =
    crate::disease_sources::components::component(
        biomcp_types::Source::MyDisease,
        biomcp_types::Source::Mondo,
        "mondo",
        crate::disease_sources::components::MONDO,
    );

pub(crate) const DOID_COMPONENT: crate::registry::Component =
    crate::disease_sources::components::component(
        biomcp_types::Source::MyDisease,
        biomcp_types::Source::Doid,
        "doid",
        crate::disease_sources::components::DOID,
    );

pub(crate) const HPO_COMPONENT: crate::registry::Component =
    crate::disease_sources::components::component(
        biomcp_types::Source::MyDisease,
        biomcp_types::Source::Hpo,
        "hpo",
        crate::disease_sources::components::HPO,
    );

pub(crate) const ORPHANET_COMPONENT: crate::registry::Component =
    crate::disease_sources::components::component(
        biomcp_types::Source::MyDisease,
        biomcp_types::Source::Orphanet,
        "orphanet",
        crate::disease_sources::components::ORPHANET,
    );

pub(crate) const OMIM_COMPONENT: crate::registry::Component =
    crate::disease_sources::components::component(
        biomcp_types::Source::MyDisease,
        biomcp_types::Source::Omim,
        "omim",
        crate::disease_sources::components::OMIM,
    );
