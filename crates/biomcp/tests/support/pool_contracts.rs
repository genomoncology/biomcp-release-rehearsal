//! Existing routine owners retain runtime IDs, destinations and asserted source rows.
//! Protein feature/member owners and oversized trial text use authored synthetic data.
pub(crate) const PATHWAY_GENES: (&str, &[&str]) = (
    "pathway-reactome/native-reference-walk",
    &[
        "option pathway genes --limit",
        "option pathway genes --offset",
        "option pathway genes --pool",
    ],
);
pub(crate) const PATHWAY_SEARCH: (&str, &[&str]) = (
    "pathway-reactome/pool-bindings-before-source-io",
    &["option search pathway --offset"],
);
pub(crate) const ORGANIZATION: (&str, &[&str]) = (
    "organization-nci/native-local-pages",
    &[
        "option search organization --offset",
        "option search organization --pool",
    ],
);
pub(crate) const BIOMARKER: (&str, &[&str]) = (
    "biomarker-nci/native-snapshot-pages",
    &[
        "option search biomarker --limit",
        "option search biomarker --offset",
        "option search biomarker --pool",
    ],
);
pub(crate) const FEATURES: (&str, &[&str]) = (
    "protein-sections/displayed-bounds-and-identity",
    &[
        "option protein features --offset",
        "option protein features --pool",
        "option protein isoforms --pool",
        "option protein isoforms --offset",
        "option protein structures --pool",
        "option protein structures --offset",
    ],
);
pub(crate) const MEMBERS: (&str, &[&str]) = (
    "protein-domains-complexes/exact-members-and-retained-pages",
    &[
        "option protein complex-members --offset",
        "option protein complex-members --pool",
        "option protein complexes --pool",
        "option protein complexes --offset",
    ],
);
pub(crate) const TRIAL: (&str, &[&str]) = (
    "trial-get/get/checked-source-continuations",
    &[
        "option get trial --offset",
        "option get trial --pool",
        "failure trial expired-pool",
    ],
);

pub(crate) const TRIAL_RESULTS: (&str, &[&str]) = (
    "trial-results/posted-values-and-event-pages",
    &["section get trial results"],
);

pub(crate) const ARTICLE_SAVES: (&str, &[&str]) = (
    "article-saves/explicit-original-files",
    &[
        "option article text --save",
        "option article text --replace",
        "option article supplement --save",
        "option article supplement --replace",
    ],
);

pub(crate) const DOMAIN_PAGES: (&str, &[&str]) = (
    "protein-domains-complexes/native-pointer-retained-walk",
    &[
        "option protein domains --pool",
        "option protein domains --offset",
    ],
);
pub(crate) const DOMAIN_TYPE: (&str, &[&str]) = (
    "protein-domains-complexes/native-types-and-fragments",
    &["option protein domains --type"],
);

// Citation window replies and oversized evidence strings are explicitly synthetic.
pub(crate) const ARTICLE_CITATIONS: (&str, &[&str]) = (
    "article-evidence/adapter-values-semantic-scholar",
    &[
        "option article citations --pool",
        "option article citations --offset",
    ],
);
pub(crate) const EVIDENCE_FRAGMENTS: (&str, &[&str]) = (
    "article-evidence/evidence/overflow-and-reference-identity",
    &[
        "option article citation-evidence --offset",
        "option article citation-evidence --paragraph-offset",
    ],
);

// CIViC uses the retained CC0 capture and literal profile/claim identities.
pub(crate) const DRUG_CIVIC: (&str, &[&str]) = (
    "drug-next-steps/native-d05-source-listed-lookup",
    &[
        "section get drug civic",
        "option get drug --civic-profile",
        "option get drug --civic-kind",
    ],
);
// Synthetic WHO rows exercise runtime revision binding, without publisher prose.
pub(crate) const WHO_REVISION: (&str, &[&str]) = (
    "diagnostic-who-search/full-walk-keeps-candidates-and-revision",
    &["option get diagnostic --bundle-revision"],
);

// Native CC0 claim page retains its source cursor and literal EID260.
pub(crate) const DRUG_CIVIC_CURSOR: (&str, &[&str]) = (
    "drug-civic/profile-evidence-next",
    &["option get drug --civic-cursor"],
);
// Synthetic repeated/empty cursors force a bad-reply boundary unavailable on demand.
pub(crate) const VARIANT_CIVIC_CURSOR: (&str, &[&str]) = (
    "variant-civic/civic-native-cursor-must-progress",
    &[
        "option get variant --civic-profile",
        "option get variant --civic-kind",
        "option get variant --civic-cursor",
    ],
);

// Original CC0 Reactome relation walks retain all81 event identities.
pub(crate) const PATHWAY_EVENTS: (&str, &[&str]) = (
    "pathway-reactome/native-event-walk",
    &[
        "option pathway events --offset",
        "option pathway events --pool",
        "option get pathway --section-limit",
        "option get pathway --section-offset",
        "option get pathway --pool",
    ],
);

// One actual local HTTP process admits the literal configured Host and refuses foreign hosts.
pub(crate) const HTTP_HOST: (&str, &[&str]) = (
    "mcp-http/host-origin-method-and-metadata-boundaries",
    &["option serve --allow-host"],
);

// This hosted stdio call asserts refusal and the ordinary unhosted health reply.
pub(crate) const HOSTED_MODE: (&str, &[&str]) =
    ("mcp-hosted/stdio/health", &["option serve --hosted"]);

// This hosted call asserts separate pacing for two source header keys.
pub(crate) const HOSTED_PACING: (&str, &[&str]) = (
    "mcp-hosted/pacing/two-header-keys",
    &["option serve --rate-share"],
);

// Operator flags override file settings and keep discovery available under tool load.
pub(crate) const HTTP_CAPACITY: (&str, &[&str]) = (
    "mcp-http/configured-capacity-and-refusals",
    &[
        "option serve --http_max_calls",
        "option serve --http_max_connections",
    ],
);

// The process settings case proves the source pacing map flag and its file precedence.
pub(crate) const SOURCE_PACING: (&str, &[&str]) = (
    "source-pacing/settings-and-ceilings",
    &["global --source_pacing"],
);
