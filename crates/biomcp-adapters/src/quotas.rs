//! Provider defaults and ceilings, with bounded primary-source policy research.
pub use crate::quota_policy::{SourcePolicy, configure};
use biomcp_fetch::{Quota, Rate, SourceKey};

// One declaration supplies transport limits, policy inventory and generated reference rows.
macro_rules! quotas {
    ($($(#[doc = $citation:literal])+ $symbol:ident = $quota:expr;)*) => {
        $(#[doc = concat!($($citation),+)]
        pub(crate) static $symbol: Quota = $quota;)*
        static POLICIES: &[SourcePolicy] = &[$(SourcePolicy { quota: &$symbol, citation: concat!($($citation),+) },)*];
        /// Every provider bucket and its published policy or bounded absence finding.
        pub fn policies() -> &'static [SourcePolicy] { POLICIES }
    };
}

// Unspecified provider concurrency retains the existing conservative local in-flight ceiling.
// Bounded absence means docs/terms and provider-domain policy searches found no numeric limit.
// EBI services also follow https://www.ebi.ac.uk/about/terms-of-use/ (revised 2026-10-14).
quotas! {
    /// Checked 2026-11-01: https://www.mygene.info/blog/update-on-mygene-down-time . Published 2019-09-18: large queries at most one per second.
    BIOTHINGS = Quota::new(SourceKey::declare("biothings"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.genenames.org/download/statistics-and-files/ . No numeric file-download limit found in download docs and
    /// provider-domain search; HGNC REST policy governs a different service.
    HGNC = Quota::new(SourceKey::declare("hgnc"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://api.oncokb.org/oncokb-website/api . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    ONCOKB = Quota::new(SourceKey::declare("oncokb"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://gnomad.broadinstitute.org/help . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    GNOMAD = Quota::new(SourceKey::declare("gnomad"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.uniprot.org/help/programmatic_access . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    UNIPROT = Quota::new(SourceKey::declare("uniprot"), Rate::per_second(2), None, 2);

    /// Checked 2026-11-01: https://reactome.org/dev/content-service . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    REACTOME = Quota::new(SourceKey::declare("reactome"), Rate::per_second(2), None, 2);

    /// Checked 2026-11-01: https://docs.civicdb.org/en/latest/api.html . Published anonymous limit: three requests per second averaged over five minutes.
    CIVIC = Quota::new(SourceKey::declare("civic"), Rate::per_second(3), None, 2);

    /// Checked 2026-11-01: https://clinicalgenome.org/docs/terms-of-use/ . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    CLINGEN = Quota::new(SourceKey::declare("clingen"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://thegencc.org/faq . No published numeric limit found in service documentation/terms and provider-domain policy search;
    /// retain conservative local ceiling.
    GENCC = Quota::new(SourceKey::declare("gencc"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.proteinatlas.org/about/download . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    HPA = Quota::new(SourceKey::declare("hpa"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://info.orcid.org/ufaqs/what-are-the-api-limits/ . Published anonymous/public limit: twelve per second. Separate daily
    /// quotas: 25000 per IP anonymous, 100000 per public client; provider enforces these.
    ORCID = Quota::new(SourceKey::declare("orcid"), Rate::per_second(12), None, 1);

    /// Checked 2026-11-01: https://www.ncbi.nlm.nih.gov/books/NBK25497/ . Published limits: three per second per IP without a key, ten per second per key.
    NCBI = Quota::new(SourceKey::declare("ncbi-eutils"), Rate::per_second(3), Some(Rate::per_second(10)), 3);

    /// Checked 2026-11-01: https://groups.google.com/a/ebi.ac.uk/g/epmc-webservices/c/xcwqYEGosp4 . Published staff policy 2022-07-11: 10 per second AND
    /// 500 per minute; evenly space 500/minute. Per-IP scope confirmed at https://groups.google.com/a/ebi.ac.uk/g/epmc-webservices/c/cZLnV1JhCj8 .
    EUROPEPMC = Quota::new(SourceKey::declare("europepmc"), Rate::per_period(500, 60), None, 1);

    /// Checked 2026-11-01: https://www.ncbi.nlm.nih.gov/research/pubtator3/api . Published API guidance: at most three requests per second.
    PUBTATOR = Quota::new(SourceKey::declare("pubtator"), Rate::per_second(3), None, 3);

    /// Checked 2026-11-01: https://pmc.ncbi.nlm.nih.gov/tools/cloud/ . No numeric S3-download limit found in cloud-service docs and provider-domain search;
    /// retain conservative local ceiling.
    PMC_OA = Quota::new(SourceKey::declare("pmc-oa"), Rate::per_second(10), None, 2);

    /// Checked 2026-11-01: https://support.icite.nih.gov/hc/en-us/articles/9513563045787-Bulk-Data-and-API . No numeric rate found in API docs and
    /// provider-domain search; batch size is not request rate.
    ICITE = Quota::new(SourceKey::declare("icite"), Rate::per_second(5), None, 4);

    /// Checked 2026-11-01: https://www.ncbi.nlm.nih.gov/research/litvar2/ . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    LITVAR = Quota::new(SourceKey::declare("litvar"), Rate::per_second(3), None, 3);

    /// Checked 2026-11-01: https://www.ncbi.nlm.nih.gov/research/litsense/ . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    LITSENSE = Quota::new(SourceKey::declare("litsense"), Rate::per_second(3), None, 3);

    /// Checked 2026-11-01: https://www.semanticscholar.org/product/api . Published introductory keyed limit: one per second. Keyless 1000 per second is
    /// shared globally, not a per-client entitlement; retain conservative local keyless ceiling.
    SEMANTIC_SCHOLAR = Quota::new(SourceKey::declare("semantic-scholar"), Rate::every_seconds(2), Some(Rate::per_second(1)), 1);

    /// Checked 2026-11-01: https://reg.genome.network/doc/ . No published numeric limit found in service documentation/terms and provider-domain policy
    /// search; retain conservative local ceiling.
    ALLELE_REGISTRY = Quota::new(SourceKey::declare("car"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://mutalyzer.nl/api/ . No published numeric limit found in service documentation/terms and provider-domain policy search;
    /// retain conservative local ceiling.
    MUTALYZER = Quota::new(SourceKey::declare("mutalyzer"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://variantvalidator.org/help/rest-api/ . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    VARIANT_VALIDATOR = Quota::new(SourceKey::declare("variantvalidator"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://clinicaltrials.gov/data-api/api . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    CTGOV = Quota::new(SourceKey::declare("ctgov"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://api.semanticscholar.org/api-docs/graph#tag/Paper-Data . No universal published numeric limit applies to article-specific
    /// publisher PDF URLs; retain conservative local ceiling. Each publisher remains authoritative.
    PUBLISHER_PDF = Quota::new(SourceKey::declare("publisher-pdf"), Rate::per_second(2), None, 1);

    /// Checked 2026-11-01: https://www.cancer.gov/syndication/api . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    NCI = Quota::new(SourceKey::declare("nci"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://docs.cbioportal.org/web-api-and-clients/ . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    CBIOPORTAL = Quota::new(SourceKey::declare("cbioportal"), Rate::per_second(2), None, 1);

    /// Checked 2026-11-01: https://www.cancerhotspots.org/ . No published numeric limit found in service documentation/terms and provider-domain policy
    /// search; retain conservative local ceiling.
    HOTSPOTS = Quota::new(SourceKey::declare("hotspots"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-04: https://www.ebi.ac.uk/gwas/docs/programmatic-access/rest-api/ . Published per-IP limits: 100 per minute AND 15 per second. Space
    /// 100/minute evenly without bursts to respect both. Allow five bounded page reads to overlap so slow replies do not serialize a result window.
    GWAS = Quota::new(SourceKey::declare("gwas"), Rate::per_period(100, 60), None, 5);

    /// Checked 2026-11-01: https://www.ebi.ac.uk/chembl/api/data/docs . No numeric data-API limit found in API docs and provider-domain search; Beaker
    /// utilities are a separate service.
    CHEMBL = Quota::new(SourceKey::declare("chembl"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://platform-docs.opentargets.org/data-access/graphql-api . No numeric limit found in GraphQL docs and provider-domain
    /// search; provider recommends downloads for systematic bulk access.
    OPENTARGETS = Quota::new(SourceKey::declare("opentargets"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://open.fda.gov/apis/authentication/ . Published limits: 240/minute per IP keyless or per key keyed. Separate daily quotas:
    /// 1000 keyless, 120000 keyed; provider enforces these and transport honors Retry-After.
    OPENFDA = Quota::new(SourceKey::declare("openfda"), Rate::per_second(4), Some(Rate::per_second(4)), 1);

    /// Checked 2026-11-01: https://www.ema.europa.eu/en/about-us/about-website/legal-notice . No published numeric limit found in service
    /// documentation/terms and provider-domain policy search; retain conservative local ceiling.
    EMA = Quota::new(SourceKey::declare("ema"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://ddinter.scbdd.com/terms/ . No published numeric limit found in service documentation/terms and provider-domain policy
    /// search; retain conservative local ceiling.
    DDINTER = Quota::new(SourceKey::declare("ddinter"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.ebi.ac.uk/ols4/help . No published numeric limit found in service documentation/terms and provider-domain policy
    /// search; retain conservative local ceiling.
    OLS = Quota::new(SourceKey::declare("ols"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://api-v3.monarchinitiative.org/docs . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    MONARCH = Quota::new(SourceKey::declare("monarch"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://seer.cancer.gov/about/using-website.html . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    SEER = Quota::new(SourceKey::declare("seer"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.ncbi.nlm.nih.gov/gtr/docs/ . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    GTR = Quota::new(SourceKey::declare("gtr-files"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.who.int/about/policies/terms-of-use . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    WHO = Quota::new(SourceKey::declare("who-ivd"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.fda.gov/about-fda/about-website/website-policies . No published numeric limit found in service documentation/terms
    /// and provider-domain policy search; retain conservative local ceiling.
    FDA_WEBSITE = Quota::new(SourceKey::declare("fda-website"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://api.cpicpgx.org/ . No published numeric limit found in service documentation/terms and provider-domain policy search;
    /// retain conservative local ceiling.
    CPIC = Quota::new(SourceKey::declare("cpic"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://api.clinpgx.org/ . Published API limit: two requests per second.
    CLINPGX = Quota::new(SourceKey::declare("clinpgx"), Rate::per_second(2), None, 1);

    /// Checked 2026-11-01: https://wonder.cdc.gov/wonder/help/wonder-api.html . Published automated-query recommendation: one query every two minutes for
    /// server recovery.
    WONDER = Quota::new(SourceKey::declare("wonder"), Rate::every_seconds(120), None, 1).refuse_wait();

    /// Checked 2026-11-01: https://www.wikipathways.org/help.html . No published numeric limit found in service documentation/terms and provider-domain
    /// policy search; retain conservative local ceiling.
    WIKIPATHWAYS = Quota::new(SourceKey::declare("wikipathways"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://hpo.jax.org/ . No published numeric limit found in service documentation/terms and provider-domain policy search; retain
    /// conservative local ceiling.
    HPO = Quota::new(SourceKey::declare("hpo"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.ebi.ac.uk/interpro/api/ . No published numeric limit found in service documentation/terms and provider-domain policy
    /// search; retain conservative local ceiling.
    INTERPRO = Quota::new(SourceKey::declare("interpro"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://www.ebi.ac.uk/complexportal/documentation . No published numeric limit found in service documentation/terms and
    /// provider-domain policy search; retain conservative local ceiling.
    COMPLEX_PORTAL = Quota::new(SourceKey::declare("complex-portal"), Rate::per_second(1), None, 1);

    /// Checked 2026-11-01: https://api.cellosaurus.org/ . No published numeric limit found in service documentation/terms and provider-domain policy
    /// search; retain conservative local ceiling.
    CELLOSAURUS = Quota::new(SourceKey::declare("cellosaurus"), Rate::per_second(2), None, 1);

    /// Checked 2026-11-01: https://www.alphagenomedocs.com/ . No fixed numeric limit published: provider says query rates vary with demand; retain
    /// conservative local ceiling.
    ALPHAGENOME = Quota::new(SourceKey::declare("alphagenome"), Rate::per_second(1), None, 1);

}
