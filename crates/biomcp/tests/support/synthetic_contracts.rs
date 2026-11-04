//! Existing mechanical trials use authored synthetic data, never native captures.
pub(crate) const GWAS_POOL: (&str, &[&str]) = (
    "gwas-search/expired-threshold-restarts-original-selectors",
    &["option search gwas --pool"],
);
pub(crate) const GWAS_ANCESTRY: (&str, &[&str]) = (
    "gwas-get/ancestry-walk-ownership-and-source-policy",
    &[
        "section get gwas ancestries",
        "option get gwas --section-limit",
        "option get gwas --section-offset",
    ],
);
pub(crate) const GWAS_ASSOCIATIONS: (&str, &[&str]) = (
    "gwas-get/loci-and-native-study-page-actions",
    &["section get gwas associations"],
);
pub(crate) const STUDY_PAGES: (&str, &[&str]) = (
    "study-get/complete-pages-and-native-counts",
    &[
        "section get study profiles",
        "section get study sample-lists",
        "section get study all",
        "option get study --section-limit",
        "option get study --section-offset",
    ],
);
pub(crate) const STUDY_FILES: (&str, &[&str]) = (
    "study-files/complete-walk-and-notice",
    &["section get study files"],
);
pub(crate) const STUDY_SEARCH: (&str, &[&str]) = (
    "study-catalog/literal-pages-and-selectors",
    &["option search study --offset", "option search study --pool"],
);
pub(crate) const CPIC_VIEW: (&str, &[&str]) = (
    "cpic-pgx/view-network-cache-replay",
    &[
        "section get pgx recommendations",
        "option get pgx --section-limit",
        "option get pgx --section-offset",
    ],
);
pub(crate) const CPIC_TABLE: (&str, &[&str]) = (
    "cpic-pgx/table-network-cache-replay",
    &["option get pgx --drug-id"],
);
pub(crate) const CPIC_GUIDELINES: (&str, &[&str]) = (
    "cpic-pgx/guidelines-network-cache-replay",
    &["section get pgx guidelines"],
);
pub(crate) const CPIC_DIPLOTYPE: (&str, &[&str]) = (
    "cpic-pgx/diplotype-network-cache-replay",
    &["section get pgx diplotype"],
);
pub(crate) const CPIC_PAIRS: (&str, &[&str]) = (
    "cpic-pgx/pairs-network-cache-replay",
    &["option search pgx --offset"],
);
pub(crate) const CLINPGX_CLINICAL: (&str, &[&str]) = (
    "clinpgx-annotations/clinical-network-cache-replay",
    &["section get pgx annotations", "option get pgx --snapshot"],
);
pub(crate) const CLINPGX_KIND: (&str, &[&str]) = (
    "clinpgx-annotations/guideline-network-cache-replay",
    &["option get pgx --kind"],
);
pub(crate) const CLINPGX_ANNOTATION: (&str, &[&str]) = (
    "clinpgx-annotations/annotation-network-cache-replay",
    &["option get pgx --annotation-id"],
);

// Label prose, keyed annotations, HPO and carried Monarch references remain synthetic.
pub(crate) const LABEL_HEADINGS: (&str, &[&str]) = (
    "drug-label/five-native-headings",
    &["section get drug label", "option get drug --label-section"],
);
pub(crate) const ONCOKB: (&str, &[&str]) = (
    "variant-oncokb/oncokb-key-policy-wire",
    &[
        "section get variant oncokb",
        "option get variant --tumor-type",
    ],
);
pub(crate) const ONCOKB_ALL: (&str, &[&str]) = (
    "variant-oncokb/oncokb-resolved-id-and-all-context",
    &["section get variant all"],
);
pub(crate) const ALPHAGENOME: (&str, &[&str]) = (
    "variant-alphagenome/alphagenome-native-groups-local-tls",
    &[
        "section get variant alphagenome",
        "option get variant --section-limit",
        "option get variant --section-offset",
    ],
);
pub(crate) const HPO_SNAPSHOT: (&str, &[&str]) = (
    "phenotype-sections/refresh-preserves-card-snapshot-and-policy",
    &["option get phenotype --snapshot"],
);
pub(crate) const MONARCH_XREFS: (&str, &[&str]) = (
    "phenotype-sections/native-classes-and-direct-links",
    &["section get phenotype xrefs"],
);

// FAERS/MedDRA and MAUDE/GMDN complete responses use authored synthetic terms.
pub(crate) const FAERS_SECTIONS: (&str, &[&str]) = (
    "adverse-event-get/synthetic-sections-and-snapshot",
    &[
        "section get adverse-event drugs",
        "section get adverse-event package-ndc",
        "option get adverse-event --snapshot",
        "option get adverse-event --section-limit",
        "option get adverse-event --section-offset",
    ],
);
pub(crate) const FAERS_IDENTIFIERS: (&str, &[&str]) = (
    "adverse-event-get/synthetic-identifier-kinds",
    &[
        "section get adverse-event unii",
        "section get adverse-event rxcui",
        "section get adverse-event product-ndc",
        "section get adverse-event spl-id",
        "section get adverse-event spl-set-id",
        "section get adverse-event application-number",
        "option get adverse-event --drug-index",
    ],
);
pub(crate) const MAUDE_SECTIONS: (&str, &[&str]) = (
    "maude-reports/synthetic-complete-section-pages",
    &[
        "section get adverse-event devices",
        "section get adverse-event narrative",
        "option get adverse-event --source",
    ],
);

// Restricted VariantValidator output uses the existing explicitly authored reply.
pub(crate) const VALIDATOR_RESOLVER: (&str, &[&str]) = (
    "variant-identity/validator-keeps-native-order-and-transcript-only-forms",
    &["option get variant --resolver"],
);
// CDC WONDER restrictions: authored tables preserve independent totals and suppression.
pub(crate) const VAERS_TABLES: (&str, &[&str]) = (
    "vaers-summary/independent-tables-retained-walk-and-suppression",
    &[
        "section search adverse-event symptoms",
        "section search adverse-event age",
        "section search adverse-event seriousness",
        "option search adverse-event --snapshot",
    ],
);
// MedDRA terms remain synthetic under the existing report-source exception.
pub(crate) const FAERS_REACTION: (&str, &[&str]) = (
    "adverse-event-search/synthetic-combined",
    &["option search adverse-event --reaction"],
);

// Mutalyzer terms require review; these three source-shaped mappings are synthetic.
pub(crate) const NORMALIZATION_PAGES: (&str, &[&str]) = (
    "variant-normalization/native-mapping-walk-is-warm",
    &[
        "option variant normalize --limit",
        "option variant normalize --offset",
    ],
);
// ClinVar submission rights remain source-specific; the existing XML is synthetic.
pub(crate) const CLINVAR_ASSERTION: (&str, &[&str]) = (
    "variant-assertions/assertions-source-wire",
    &["option variant assertions --assertion"],
);
pub(crate) const CLINVAR_PAGES: (&str, &[&str]) = (
    "variant-assertions/assertions-native-row-walk-preserves-unmapped",
    &[
        "option variant assertions --limit",
        "option variant assertions --offset",
    ],
);
// HPO's custom license remains explicit; Monarch association labels are synthetic.
pub(crate) const MONARCH_PHENOTYPES: (&str, &[&str]) = (
    "disease-monarch/phenotype-evidence-and-independent-native-qualifiers",
    &["section get disease phenotypes"],
);

// MyVariant component terms are restricted; authored ambiguous rows retain exact IDs.
pub(crate) const POPULATION_SELECTORS: (&str, &[&str]) = (
    "variant-population/population-ambiguous-ids-retain-source-alleles",
    &[
        "option variant population --resolver",
        "option variant population --build",
        "option variant population --limit",
    ],
);

// Cancer Hotspots redistribution terms are unverified; the existing counts are synthetic.
pub(crate) const HOTSPOTS: (&str, &[&str]) = (
    "variant-hotspots/hotspots-retains-native-transcript-and-counts",
    &["section get variant hotspots"],
);
