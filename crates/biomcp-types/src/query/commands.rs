//! The single declaration of checked commands, schema identities and rewrites.
/// Apply a consumer to the complete command declaration.
#[macro_export]
macro_rules! command_types {
    ($consumer:ident) => {
        $consumer! {
            rewrites { Direct, GetPgx, GetDrug, GetGwas, SearchGwas, SearchCellLine, SearchAuthor, GetPhenotype }
            /// Find native NCI biomarker concepts.
            SearchBiomarker(BiomarkerSearch) => (
                "search", "biomarker", BiomarkerSearch, "biomarker-search", "biomcp.biomarker-search/2", BiomarkerSearch, "biomarker-search",
                "biomcp.biomarker-search/2", "biomcp.biomarker-search/2", 0, Direct
            );
            /// Search native NCI biomarker rows by exact NCIt code.
            GetBiomarker(BiomarkerGet) => (
                "get", "biomarker", BiomarkerGet, "biomarker-get", "biomcp.biomarker-get/1", BiomarkerGet, "biomarker-get", "biomcp.biomarker-get/1",
                "biomcp.biomarker-get/1", 1, Direct
            );
            /// Find NCI organization names without assigning an identity.
            SearchOrganization(OrganizationSearch) => (
                "search", "organization", OrganizationSearch, "organization-search", "biomcp.organization-search/2", OrganizationSearch,
                "organization-search", "biomcp.organization-search/2", "biomcp.organization-search/2", 4, Direct
            );
            /// Discover native author candidates without identity reconciliation.
            SearchAuthor(AuthorSearch) => (
                "search", "author", AuthorSearch, "author-search-query", "biomcp.author-search-query/3", AuthorSearch, "author-search-query",
                "biomcp.author-search-query/3", "biomcp.author-search-query/3", 3, SearchAuthor
            );
            /// Page one exact provider record’s native papers.
            AuthorPapers(AuthorPapersQuery) => (
                "author", "papers", AuthorPapersQuery, "author-papers-query", "biomcp.author-papers-query/2", AuthorPapersQuery,
                "author-papers-query", "biomcp.author-papers-query/2", "biomcp.author-papers-query/2", 2, Direct
            );
            /// Read exact provider author cards in input order.
            GetAuthor(AuthorGet) => (
                "get", "author", AuthorGet, "author-get-query", "biomcp.author-get-query/2", AuthorGet, "author-get-query",
                "biomcp.author-get-query/2", "biomcp.author-get-query/2", 5, Direct
            );
            /// Read the native assay total for one exact ChEMBL cell.
            CellLineAssayCount(CellLineAssayCountQuery) => (
                "cell-line", "assay-count", CellLineAssayCountQuery, "cell-line-assay-count-query", "biomcp.cell-line-assay-count-query/1",
                CellLineAssayCountQuery, "cell-line-assay-count-query", "biomcp.cell-line-assay-count-query/1",
                "biomcp.cell-line-assay-count-query/1", 6, Direct
            );
            /// Discover unchanged literal Cellosaurus candidates.
            SearchCellLine(CellLineSearchCommand) => (
                "search", "cell-line", CellLineSearchCommand, "cell-line-search-command", "biomcp.cell-line-search-command/1", CellLineSearchCommand,
                "cell-line-search-command", "biomcp.cell-line-search-command/1", "biomcp.cell-line-search-command/1", 7, SearchCellLine
            );
            /// Read an exact accession or verified qualified source identity.
            GetCellLine(CellLineGet) => (
                "get", "cell-line", CellLineGet, "cell-line-get-query", "biomcp.cell-line-get-query/1", CellLineGet, "cell-line-get-query",
                "biomcp.cell-line-get-query/1", "biomcp.cell-line-get-query/1", 8, Direct
            );
            /// Search one native GWAS Catalog association page.
            SearchGwas(GwasSearch) => (
                "search", "gwas", GwasSearchCommand, "gwas-search", "biomcp.gwas-search/4", GwasSearchCommand, "gwas-search", "biomcp.gwas-search/4",
                "biomcp.gwas-search/4", 10, SearchGwas
            );
            /// Read one exact association and its native loci.
            GetGwasAssociation(GwasAssociationGet) => (
                "get", "gwas", GwasGetCommand, "gwas-get", "biomcp.gwas-get/2", GwasGetCommand, "gwas-get", "biomcp.gwas-get/2", "biomcp.gwas-get/2",
                9, GetGwas
            );
            /// Read one exact study and its native relations.
            GetGwasStudy(GwasStudyGet) => (
                "get", "gwas", GwasGetCommand, "gwas-get", "biomcp.gwas-get/2", GwasGetCommand, "gwas-get", "biomcp.gwas-get/2", "biomcp.gwas-get/2",
                9, GetGwas
            );
            /// Read native InterPro matches.
            ProteinDomains(ProteinDomainsQuery) => (
                "protein", "domains", ProteinDomainsQuery, "protein-domains-query", "biomcp.protein-domains-query/2", ProteinDomainsQuery,
                "protein-domains-query", "biomcp.protein-domains-query/2", "biomcp.protein-domains-query/2", 13, Direct
            );
            /// Read exact participant-admitted complex headers.
            ProteinComplexes(ProteinRelationQuery) => (
                "protein", "complexes", ProteinRelationQuery, "protein-relation-query", "biomcp.protein-relation-query/2", ProteinRelationQuery,
                "protein-relation-query", "biomcp.protein-relation-query/2", "biomcp.protein-relation-query/2", 14, Direct
            );
            /// Read native participants under an exact protein anchor.
            ProteinComplexMembers(ProteinComplexMembersQuery) => (
                "protein", "complex-members", ProteinComplexMembersQuery, "protein-complex-members-query", "biomcp.protein-complex-members-query/2",
                ProteinComplexMembersQuery, "protein-complex-members-query", "biomcp.protein-complex-members-query/2",
                "biomcp.protein-complex-members-query/2", 15, Direct
            );
            /// Read exact native feature rows with checked residue selectors.
            ProteinFeatures(ProteinFeaturesQuery) => (
                "protein", "features", ProteinFeaturesQuery, "protein-features-query", "biomcp.protein-features-query/2", ProteinFeaturesQuery,
                "protein-features-query", "biomcp.protein-features-query/2", "biomcp.protein-features-query/2", 11, Direct
            );
            /// Read native alternative products without constructing sequences.
            ProteinIsoforms(ProteinRelationQuery) => (
                "protein", "isoforms", ProteinRelationQuery, "protein-relation-query", "biomcp.protein-relation-query/2", ProteinRelationQuery,
                "protein-relation-query", "biomcp.protein-relation-query/2", "biomcp.protein-relation-query/2", 12, Direct
            );
            /// Read source PDB references without residue coverage inference.
            ProteinStructures(ProteinRelationQuery) => (
                "protein", "structures", ProteinRelationQuery, "protein-relation-query", "biomcp.protein-relation-query/2", ProteinRelationQuery,
                "protein-relation-query", "biomcp.protein-relation-query/2", "biomcp.protein-relation-query/2", 16, Direct
            );
            /// Read one exact source sequence without alias repair.
            ProteinSequence(ProteinSequenceQuery) => (
                "protein", "sequence", ProteinSequenceQuery, "protein-sequence-query", "biomcp.protein-sequence-query/1", ProteinSequenceQuery,
                "protein-sequence-query", "biomcp.protein-sequence-query/1", "biomcp.protein-sequence-query/1", 17, Direct
            );
            /// Discover reviewed human UniProt candidates without selecting one.
            SearchProtein(ProteinSearch) => (
                "search", "protein", ProteinSearch, "protein-search", "biomcp.protein-search/2", ProteinSearch, "protein-search",
                "biomcp.protein-search/2", "biomcp.protein-search/2", 18, Direct
            );
            /// Read exact protein cards or all HGNC-listed protein candidates.
            GetProtein(ProteinGet) => (
                "get", "protein", ProteinGet, "protein-get", "biomcp.protein-get/2", ProteinGet, "protein-get", "biomcp.protein-get/2",
                "biomcp.protein-get/2", 19, Direct
            );
            /// Discover literal HP term candidates in publisher order.
            SearchPhenotype(PhenotypeSearch) => (
                "search", "phenotype", PhenotypeSearch, "phenotype-search-query", "biomcp.phenotype-search-query/1", PhenotypeSearch,
                "phenotype-search-query", "biomcp.phenotype-search-query/1", "biomcp.phenotype-search-query/1", 20, Direct
            );
            /// Read one independent exact HPO or Monarch term card.
            GetPhenotype(PhenotypeGet) => (
                "get", "phenotype", PhenotypeGet, "phenotype-get", "biomcp.phenotype-get/2", PhenotypeGet, "phenotype-get", "biomcp.phenotype-get/2",
                "biomcp.phenotype-get/2", 21, GetPhenotype
            );
            /// Discover source-native human pathway headers.
            SearchPathway(PathwaySearch) => (
                "search", "pathway", PathwaySearch, "pathway-search", "biomcp.pathway-search/4", PathwaySearch, "pathway-search",
                "biomcp.pathway-search/4", "biomcp.pathway-search/4", 22, Direct
            );
            /// Read an exact human pathway card or summary.
            GetPathway(PathwayGet) => (
                "get", "pathway", PathwayGet, "pathway-get", "biomcp.pathway-get/3", PathwayGet, "pathway-get", "biomcp.pathway-get/3",
                "biomcp.pathway-get/3", 23, Direct
            );
            /// Page retained exact source membership references.
            PathwayGenes(PathwayGenesQuery) => (
                "pathway", "genes", PathwayGenesQuery, "pathway-genes-query", "biomcp.pathway-genes-query/3", PathwayGenesQuery,
                "pathway-genes-query", "biomcp.pathway-genes-query/3", "biomcp.pathway-genes-query/3", 24, Direct
            );
            /// Page retained source-contained event references.
            PathwayEvents(PathwayEventsQuery) => (
                "pathway", "events", PathwayEventsQuery, "pathway-events-query", "biomcp.pathway-events-query/2", PathwayEventsQuery,
                "pathway-events-query", "biomcp.pathway-events-query/2", "biomcp.pathway-events-query/2", 25, Direct
            );
            /// Discover literal study catalog metadata.
            SearchStudy(StudySearch) => (
                "search", "study", StudySearch, "study-search", "biomcp.study-search/2", StudySearch, "study-search", "biomcp.study-search/2",
                "biomcp.study-search/2", 26, Direct
            );
            /// Read exact study metadata.
            GetStudy(StudyGet) => (
                "get", "study", StudyGet, "study-get", "biomcp.study-get/1", StudyGet, "study-get", "biomcp.study-get/1", "biomcp.study-get/1", 27,
                Direct
            );
            /// Discover source-native CPIC gene pairs.
            SearchPgx(PgxSearch) => (
                "search", "pgx", PgxSearch, "pgx-search", "biomcp.pgx-search/3", PgxSearch, "pgx-search", "biomcp.pgx-search/3",
                "biomcp.pgx-search/3", 28, Direct
            );
            /// Read an explicit CPIC gene card or source section.
            GetPgx(PgxGet) => (
                "get", "pgx", PgxGet, "pgx-get", "biomcp.pgx-get/3", PgxGetCommand, "pgx-get-command", "biomcp.pgx-get-command/3",
                "biomcp.pgx-get/3", 29, GetPgx
            );
            /// Read one exact ClinPGx clinical annotation accession.
            GetPgxAnnotation(PgxAnnotationGet) => (
                "get", "pgx", PgxAnnotationGet, "pgx-annotation-get", "biomcp.pgx-annotation-get/2", PgxGetCommand, "pgx-get-command",
                "biomcp.pgx-get-command/3", "biomcp.pgx-get-command/3", 29, GetPgx
            );
            /// Discover native diagnostic laboratory tests.
            SearchDiagnostic(DiagnosticSearch) => (
                "search", "diagnostic", DiagnosticSearch, "diagnostic-search", "biomcp.diagnostic-search/3", DiagnosticSearch, "diagnostic-search",
                "biomcp.diagnostic-search/3", "biomcp.diagnostic-search/3", 30, Direct
            );
            /// Read an exact source diagnostic record or relation page.
            GetDiagnostic(DiagnosticGet) => (
                "get", "diagnostic", DiagnosticGet, "diagnostic-get", "biomcp.diagnostic-get/2", DiagnosticGet, "diagnostic-get",
                "biomcp.diagnostic-get/2", "biomcp.diagnostic-get/2", 31, Direct
            );
            /// Search source-native adverse event reports.
            SearchAdverseEvent(AdverseEventSearch) => (
                "search", "adverse-event", AdverseEventSearch, "adverse-event-search", "biomcp.adverse-event-search/4", AdverseEventSearchCommand,
                "adverse-event-search-command", "biomcp.adverse-event-search-command/1", "biomcp.adverse-event-search/4", 32, Direct
            );
            /// Read one exact source report, without source probing.
            GetAdverseEvent(AdverseEventGet) => (
                "get", "adverse-event", AdverseEventGet, "adverse-event-get", "biomcp.adverse-event-get/2", AdverseEventGet, "adverse-event-get",
                "biomcp.adverse-event-get/2", "biomcp.adverse-event-get/2", 33, Direct
            );
            /// Discover one source-native disease candidate page.
            SearchDisease(DiseaseSearch) => (
                "search", "disease", DiseaseSearch, "disease-search", "biomcp.disease-search/1", DiseaseSearch, "disease-search",
                "biomcp.disease-search/1", "biomcp.disease-search/1", 34, Direct
            );
            /// Read an exact source disease concept or native statistic page.
            GetDisease(DiseaseGet) => (
                "get", "disease", DiseaseGet, "disease-get", "biomcp.disease-get/3", DiseaseGet, "disease-get", "biomcp.disease-get/3",
                "biomcp.disease-get/3", 35, Direct
            );
            /// Discover literal native drug candidates.
            SearchDrug(DrugSearch) => (
                "search", "drug", DrugSearch, "drug-search", "biomcp.drug-search/4", DrugSearch, "drug-search", "biomcp.drug-search/4",
                "biomcp.drug-search/4", 36, Direct
            );
            /// Read one exact source drug identity.
            GetDrug(DrugGet) => (
                "get", "drug", DrugGet, "drug-get", "biomcp.drug-get/3", DrugGetCommand, "drug-get-command", "biomcp.drug-get-command/2",
                "biomcp.drug-get-command/2", 37, GetDrug
            );
            /// Read native WHO prequalification rows by exact reference.
            GetWhoMedicine(DrugWhoGet) => (
                "get", "drug", DrugWhoGet, "drug-who-get", "biomcp.drug-who-get/2", DrugGetCommand, "drug-get-command", "biomcp.drug-get-command/2",
                "biomcp.drug-get-command/2", 37, GetDrug
            );
            /// Read source-native variant assertions.
            VariantAssertions(VariantAssertionsGet) => (
                "variant", "assertions", VariantAssertionsGet, "variant-assertions", "biomcp.variant-assertions/5", VariantAssertionsGet,
                "variant-assertions", "biomcp.variant-assertions/5", "biomcp.variant-assertions/5", 52, Direct
            );
            /// Read source ancestry observations.
            VariantPopulation(VariantPopulationQuery) => (
                "variant", "population", VariantPopulationQuery, "variant-population", "biomcp.variant-population/5", VariantPopulationQuery,
                "variant-population", "biomcp.variant-population/5", "biomcp.variant-population/5", 54, Direct
            );
            /// Observe an exact allele in an explicitly selected cohort.
            VariantRecurrence(VariantRecurrenceQuery) => (
                "variant", "recurrence", VariantRecurrenceQuery, "variant-recurrence-query", "biomcp.variant-recurrence-query/6",
                VariantRecurrenceQuery, "variant-recurrence-query", "biomcp.variant-recurrence-query/6", "biomcp.variant-recurrence-query/6", 56,
                Direct
            );
            /// Send unchanged HGVS to explicit normalization services.
            VariantNormalize(VariantNormalize) => (
                "variant", "normalize", VariantNormalize, "variant-normalize", "biomcp.variant-normalize/5", VariantNormalize, "variant-normalize",
                "biomcp.variant-normalize/5", "biomcp.variant-normalize/5", 57, Direct
            );
            /// Read a published variant identifier.
            GetVariant(VariantGet) => (
                "get", "variant", VariantGet, "variant-get", "biomcp.variant-get/7", VariantGet, "variant-get", "biomcp.variant-get/7",
                "biomcp.variant-get/7", 53, Direct
            );
            /// Discover source-native variant candidates.
            SearchVariant(VariantSearch) => (
                "search", "variant", VariantSearch, "variant-search", "biomcp.variant-search/5", VariantSearch, "variant-search",
                "biomcp.variant-search/5", "biomcp.variant-search/5", 55, Direct
            );
            /// Save an exact registry document to an explicit destination.
            TrialDocument(TrialDocumentGet) => (
                "trial", "document", TrialDocumentGet, "trial-document", "biomcp.trial-document/2", TrialDocumentGet, "trial-document",
                "biomcp.trial-document/2", "biomcp.trial-document/2", 51, Direct
            );
            /// Retrieve exact trial cards and source protocol sections.
            GetTrial(TrialGet) => (
                "get", "trial", TrialGet, "trial-get", "biomcp.trial-get/2", TrialGet, "trial-get", "biomcp.trial-get/2", "biomcp.trial-get/2", 50,
                Direct
            );
            /// Search ClinicalTrials.gov without local screening.
            SearchTrial(TrialSearch) => (
                "search", "trial", TrialSearch, "trial-search", "biomcp.trial-search/6", TrialSearch, "trial-search", "biomcp.trial-search/6",
                "biomcp.trial-search/6", 49, Direct
            );
            /// Save a publisher PDF to an explicit destination.
            ArticlePdf(ArticlePdf) => (
                "article", "pdf", ArticlePdf, "article-pdf", "biomcp.article-pdf/3", ArticlePdf, "article-pdf", "biomcp.article-pdf/3",
                "biomcp.article-pdf/3", 43, Direct
            );
            /// Read source citation evidence.
            ArticleEvidence(ArticleEvidence) => (
                "article", "citation-evidence", ArticleEvidence, "article-evidence", "biomcp.article-evidence/3", ArticleEvidence,
                "article-evidence", "biomcp.article-evidence/3", "biomcp.article-evidence/3", 47, Direct
            );
            /// Source-native paper recommendations.
            ArticleRecommend(ArticleRecommend) => (
                "article", "recommendations", ArticleRecommend, "article-recommend", "biomcp.article-recommend/3", ArticleRecommend,
                "article-recommend", "biomcp.article-recommend/3", "biomcp.article-recommend/3", 48, Direct
            );
            /// Read a licensed supplementary file.
            ArticleSupplement(ArticleSupplement) => (
                "article", "supplement", ArticleSupplement, "article-supplement", "biomcp.article-supplement/3", ArticleSupplement,
                "article-supplement", "biomcp.article-supplement/3", "biomcp.article-supplement/3", 42, Direct
            );
            /// Read licensed PMC body text.
            ArticleText(ArticleText) => (
                "article", "text", ArticleText, "article-text", "biomcp.article-text/4", ArticleText, "article-text", "biomcp.article-text/4",
                "biomcp.article-text/4", 44, Direct
            );
            /// Papers citing one PMID.
            ArticleCitations(ArticleEdges) => (
                "article", "citations", ArticleEdges, "article-edges", "biomcp.article-edges/4", ArticleEdges, "article-edges",
                "biomcp.article-edges/4", "biomcp.article-edges/4", 45, Direct
            );
            /// Papers referenced by one PMID.
            ArticleReferences(ArticleEdges) => (
                "article", "references", ArticleEdges, "article-edges", "biomcp.article-edges/4", ArticleEdges, "article-edges",
                "biomcp.article-edges/4", "biomcp.article-edges/4", 46, Direct
            );
            /// Retrieve PubMed article cards.
            GetArticle(ArticleGet) => (
                "get", "article", ArticleGet, "article-get", "biomcp.article-get/3", ArticleGet, "article-get", "biomcp.article-get/3",
                "biomcp.article-get/3", 41, Direct
            );
            /// Search PubMed article identities.
            SearchArticle(ArticleSearch) => (
                "search", "article", ArticleSearch, "article-search", "biomcp.article-search/3", ArticleSearch, "article-search",
                "biomcp.article-search/3", "biomcp.article-search/3", 40, Direct
            );
            /// Get gene identities and annotations.
            GetGene(GeneGet) => (
                "get", "gene", GeneGet, "gene-get", "biomcp.gene-get/2", GeneGet, "gene-get", "biomcp.gene-get/2", "biomcp.gene-get/2", 38, Direct
            );
            /// Search gene headers.
            SearchGene(GeneSearch) => (
                "search", "gene", GeneSearch, "gene-search", "biomcp.gene-search/2", GeneSearch, "gene-search", "biomcp.gene-search/2",
                "biomcp.gene-search/2", 39, Direct
            );
        }
    };
}

macro_rules! commands {
    (rewrites { $($rewrite:ident),+ } $( $(#[$doc:meta])* $variant:ident($query:ident) =>
        ($verb:literal, $entity:literal, $schema_type:ident, $stem:literal, $schema:literal,
         $grammar_type:ident, $grammar_stem:literal, $grammar_schema:literal, $spelling_schema:literal, $order:literal, $via:ident); )+) => {
        /// One checked query. Housekeeping actions are separate.
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "command", rename_all = "kebab-case")]
        #[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
        pub enum Request { $( $(#[$doc])* $variant(crate::$query), )+ }

        /// Checked translations between schema fields and public command arguments.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Rewrite { $( #[doc = stringify!($rewrite)] $rewrite, )+ }

        impl Request {
            /// Public command path.
            pub fn path(&self) -> (&'static str, &'static str) {
                match self { $(Self::$variant(_) => ($verb, $entity),)+ }
            }
            /// Embedded schema used when spelling this checked query.
            pub fn schema(&self) -> &'static str {
                match self { $(Self::$variant(_) => $spelling_schema,)+ }
            }
            /// Translation required at the public command boundary.
            pub fn rewrite(&self) -> Rewrite {
                match self { $(Self::$variant(_) => Rewrite::$via,)+ }
            }

        }
        /// Every command row, including shared rewritten paths.
        pub const COMMANDS: &[CommandSpec] = &[$(CommandSpec {
            verb: $verb, entity: $entity, schema: $grammar_schema, via: Rewrite::$via, order: $order,
        },)+];
    };
}

command_types!(commands);

/// One public path and its schema-owned grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    /// Public root command.
    pub verb: &'static str,
    /// Public child command.
    pub entity: &'static str,
    /// Embedded grammar schema identity.
    pub schema: &'static str,
    /// Checked translation for this path.
    pub via: Rewrite,
    /// Stable public presentation order.
    pub order: usize,
}
/// Unique public paths in stable presentation order.
pub fn command_paths() -> Vec<(&'static str, &'static str, &'static str)> {
    let mut rows: Vec<_> = COMMANDS.iter().collect();
    rows.sort_by_key(|command| command.order);
    rows.dedup_by_key(|command| command.order);
    rows.into_iter()
        .map(|command| (command.verb, command.entity, command.schema))
        .collect()
}
