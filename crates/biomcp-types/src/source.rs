use biodata::canonical_enum;

canonical_enum! {
    /// A source of BioMCP data, identified by its canonical short name.
    Source, SourceUnknown {
        /// HGNC.
        Hgnc => "hgnc",
        /// MyGene.info.
        MyGene => "mygene",
        /// Reactome.
        Reactome => "reactome",
        /// WikiPathways published pathway catalogs.
        WikiPathways => "wikipathways",
        /// Human Protein Atlas.
        Hpa => "hpa",
        /// UniProt.
        UniProt => "uniprot",
        /// Integrated InterPro entries.
        InterPro => "interpro",
        /// Complex Portal.
        ComplexPortal => "complex-portal",
        /// gnomAD.
        Gnomad => "gnomad",
        /// ClinVar component source policy.
        ClinVar => "clinvar",
        /// ClinGen Evidence Repository.
        Erepo => "erepo",
        /// GWAS Catalog v2.
        Gwas => "gwas",
        /// ClinGen.
        ClinGen => "clingen",
        /// GenCC submitter assessments.
        Gencc => "gencc",
        /// PubMed.
        PubMed => "pubmed",
        /// Europe PMC.
        EuropePmc => "europepmc",
        /// NCBI LitVar2.
        LitVar => "litvar",
        /// NCBI LitSense2.
        LitSense => "litsense",
        /// Semantic Scholar.
        SemanticScholar => "semantic-scholar",
        /// ORCID public registry.
        Orcid => "orcid",
        /// PMC Open Access bucket.
        PmcOa => "pmc-oa",
        /// NIH iCite.
        Icite => "icite",
        /// PubTator.
        PubTator => "pubtator",
        /// ClinicalTrials.gov.
        ClinicalTrialsGov => "ctgov",
        /// NCI Clinical Trials Search.
        Nci => "nci",
        /// CIViC.
        Civic => "civic",
        /// Cancer Hotspots residue recurrence.
        CancerHotspots => "hotspots",
        /// AlphaGenome.
        AlphaGenome => "alphagenome",
        /// OncoKB.
        Oncokb => "oncokb",
        /// MyVariant.info aggregation.
        MyVariant => "myvariant",
        /// ClinGen Allele Registry.
        AlleleRegistry => "car",
        /// Mutalyzer.
        Mutalyzer => "mutalyzer",
        /// VariantValidator.
        VariantValidator => "variantvalidator",
        /// MyChem.info aggregation.
        Mychem => "mychem",
        /// ChEMBL native records and carried component.
        Chembl => "chembl",
        /// Open Targets.
        Opentargets => "opentargets",
        /// openFDA SPL labels.
        OpenfdaLabel => "openfda-label",
        /// Drugs@FDA application records.
        Drugsfda => "drugsfda",
        /// European Medicines Agency.
        Ema => "ema",
        /// WHO finished pharmaceutical products.
        WhoPq => "who-pq",
        /// DDInter interaction records.
        Ddinter => "ddinter",
        /// MyChem DrugBank Open identity subset only.
        DrugbankOpen => "drugbank-open",
        /// MyChem DrugCentral component.
        Drugcentral => "drugcentral",
        /// MyChem Guide to Pharmacology component.
        Gtopdb => "gtopdb",
        /// MyChem UNII identities.
        Unii => "unii",
        /// MyChem NDC package identities.
        Ndc => "ndc",
        /// MyChem openFDA cross-references only.
        OpenfdaXref => "openfda-xref",
        /// Ontology Lookup Service.
        Ols => "ols",
        /// MyDisease.info aggregation.
        MyDisease => "mydisease",
        /// Monarch associations.
        Monarch => "monarch",
        /// SEER Explorer.
        Seer => "seer",
        /// MONDO published ontology content.
        Mondo => "mondo",
        /// Disease Ontology published content.
        Doid => "doid",
        /// NCI Thesaurus ontology content.
        Ncit => "ncit",
        /// HPO-published annotations.
        Hpo => "hpo",
        /// Reviewed Orphadata Science content.
        Orphanet => "orphanet",
        /// Separately licensed OMIM payload only.
        Omim => "omim",
        /// openFDA FAERS drug adverse event reports.
        Faers => "faers",
        /// CDC WONDER vaccine report summaries.
        Vaers => "vaers",
        /// openFDA MAUDE device event reports.
        Maude => "maude",
        /// Source-reported MedDRA reaction labels and versions.
        Meddra => "meddra",
        /// Source-carried GMDN device terminology.
        Gmdn => "gmdn",
        /// NCBI Genetic Testing Registry public reports.
        Gtr => "gtr",
        /// WHO prequalified in vitro diagnostics.
        WhoIvd => "who-ivd",
        /// FDA companion diagnostic device tables.
        FdaCdx => "fda-cdx",
        /// openFDA premarket approval submissions.
        FdaPma => "fda-pma",
        /// FDA native 510(k) clearance records.
        Fda510k => "fda-510k",
        /// openFDA medical device product recall records.
        FdaDeviceRecall => "fda-device-recall",
        /// FDA drug enforcement recall records.
        FdaDrugRecall => "fda-drug-recall",
        /// Clinical Pharmacogenetics Implementation Consortium.
        Cpic => "cpic",
        /// ClinPGx pharmacogenomics annotations.
        ClinPgx => "clinpgx",
        /// Public cBioPortal with study-specific reuse terms.
        Cbioportal => "cbioportal",
        /// Cellosaurus cell line identity.
        Cellosaurus => "cellosaurus",
    }
}

canonical_enum! {
    /// Why a source could not supply data.
    Reason, ReasonUnknown {
        /// The requested record was not found.
        NotFound => "not-found",
        /// The public bucket does not hold an open text record.
        NotOpen => "not-open",
        /// Publisher response is not a PDF.
        NotPdf => "not-pdf",
        /// The local reader does not support the file or container format.
        Unsupported => "unsupported",
        /// The response exceeds an explicit reader resource bound.
        TooLarge => "too-large",
        /// The source limited the request rate.
        RateLimited => "rate-limited",
        /// The local source quota refused admission without a network request.
        LocalQuota => "local-quota",
        /// The source was unavailable.
        Unavailable => "unavailable",
        /// Platform or configured certificate verification failed.
        Tls => "tls",
        /// Connecting to or negotiating the configured proxy failed.
        Proxy => "proxy",
        /// The source rejected the request.
        Rejected => "rejected",
        /// The reply could not be used.
        BadReply => "bad-reply",
        /// The request exhausted its deadline.
        DeadlineExceeded => "deadline-exceeded",
        /// Network access was disabled.
        Offline => "offline",
        /// A required credential was absent.
        NoKey => "no-key",
        /// The source was disabled.
        Disabled => "disabled",
        /// No recording matched the request.
        ReplayMiss => "replay-miss",
        /// The source data was being synchronized.
        Syncing => "syncing",
        /// Identity lacks the identifier required to join this source.
        NoJoinId => "no-join-id",
    }
}

biodata::wire_enum! {
    /// Reviewed source-use category, without inferring rights in individual records.
    #[derive(PartialOrd, Ord)]
    Use {
        /// Public-domain or unrestricted source data.
        Open => "open",
        /// Attribution is required.
        Attribution => "attribution",
        /// Commercial use requires separate permission.
        Noncommercial => "noncommercial",
        /// Separate publisher permission is required.
        Licensed => "licensed",
        /// Reuse rights are owned by each individual record.
        PerRecord => "per-record",
    }
}

impl Source {
    /// Preserve this consumer's canonical source spelling in neutral provenance.
    #[expect(
        clippy::expect_used,
        reason = "The closed source declaration contains only nonempty canonical spellings"
    )]
    pub fn text(self) -> biodata::Text {
        biodata::Text::new(self.into()).expect("canonical source spelling is nonempty")
    }
}
