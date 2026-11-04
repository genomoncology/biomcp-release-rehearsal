//! Source facts shared by discovery, health and commands.
use crate::decl::{Guidance, Probe, SourceDecl};
pub(crate) static DECL: SourceDecl = SourceDecl {
    spec: &super::SPEC,
    display: "MyChem.info",
    registry_order: Some(44),
    probe: Probe::Get("/metadata", &[]),
    components: &[
        DRUGBANK_COMPONENT,
        CHEMBL_COMPONENT,
        DRUGCENTRAL_COMPONENT,
        GTOPDB_COMPONENT,
        UNII_COMPONENT,
        NDC_COMPONENT,
        OPENFDA_COMPONENT,
    ],
    auxiliary: &[],
    bundles: &[],
    guidance: Guidance {
        deadline: None,
        quota: None,
    },
    key: None,
};

pub(crate) static DRUGBANKOPEN: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::DrugbankOpen,
        &super::SPEC,
        DRUGBANK_COMPONENT.terms,
    ),
    display: "DrugBank Open",
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

pub(crate) static DRUGCENTRAL: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Drugcentral,
        &super::SPEC,
        DRUGCENTRAL_COMPONENT.terms,
    ),
    display: "DrugCentral",
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

pub(crate) static GTOPDB: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Gtopdb,
        &super::SPEC,
        GTOPDB_COMPONENT.terms,
    ),
    display: "Guide to Pharmacology",
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

pub(crate) static UNII: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Unii,
        &super::SPEC,
        UNII_COMPONENT.terms,
    ),
    display: "FDA UNII",
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

pub(crate) static NDC: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::Ndc,
        &super::SPEC,
        NDC_COMPONENT.terms,
    ),
    display: "FDA NDC",
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

pub(crate) static OPENFDAXREF: SourceDecl = SourceDecl {
    spec: &crate::decl::component_spec(
        biomcp_types::Source::OpenfdaXref,
        &super::SPEC,
        OPENFDA_COMPONENT.terms,
    ),
    display: "openFDA cross-reference",
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

pub(crate) const DRUGBANK_COMPONENT: crate::registry::Component =
    crate::drug_sources::components::component(
        "drugbank",
        biomcp_types::Source::DrugbankOpen,
        biomcp_fetch::Terms {
            name: "DrugBank Open identity subset CC0 (captured metadata)",
            url: "https://go.drugbank.com/releases/latest#open-data",
            usage: biomcp_types::Use::Open,
        },
    );

pub(crate) const CHEMBL_COMPONENT: crate::registry::Component =
    crate::drug_sources::components::component(
        "chembl",
        biomcp_types::Source::Chembl,
        crate::drug_sources::CHEMBL_TERMS,
    );

pub(crate) const DRUGCENTRAL_COMPONENT: crate::registry::Component =
    crate::drug_sources::components::component(
        "drugcentral",
        biomcp_types::Source::Drugcentral,
        biomcp_fetch::Terms {
            name: "DrugCentral CC BY-SA 4.0",
            url: "http://drugcentral.org/privacy",
            usage: biomcp_types::Use::Attribution,
        },
    );

pub(crate) const GTOPDB_COMPONENT: crate::registry::Component =
    crate::drug_sources::components::component(
        "gtopdb",
        biomcp_types::Source::Gtopdb,
        biomcp_fetch::Terms {
            name: "GtoPdb CC BY-SA 4.0",
            url: "https://www.guidetopharmacology.org/download.jsp",
            usage: biomcp_types::Use::Attribution,
        },
    );

pub(crate) const UNII_COMPONENT: crate::registry::Component =
    crate::drug_sources::components::component(
        "unii",
        biomcp_types::Source::Unii,
        biomcp_fetch::Terms {
            name: "UNII public domain (captured metadata)",
            url: "https://www.nlm.nih.gov/web_policies.html",
            usage: biomcp_types::Use::Open,
        },
    );

pub(crate) const NDC_COMPONENT: crate::registry::Component =
    crate::drug_sources::components::component(
        "ndc",
        biomcp_types::Source::Ndc,
        biomcp_fetch::Terms {
            name: "NDC public domain (captured metadata)",
            url: "https://www.fda.gov/about-fda/about-website/website-policies",
            usage: biomcp_types::Use::Open,
        },
    );

pub(crate) const OPENFDA_COMPONENT: crate::registry::Component =
    crate::drug_sources::components::component(
        "openfda",
        biomcp_types::Source::OpenfdaXref,
        crate::drug_sources::FDA_TERMS,
    );
