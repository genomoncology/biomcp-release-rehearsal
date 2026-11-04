#![doc = include_str!("../README.md")]
// The fixed migration-message schema expands one enum of wire literals.
#![recursion_limit = "256"]
use biodata::{
    AccessionBase, Attribution, Build, Chromosome, Concept, Curie, Cytoband, Doi, EnsemblGeneId,
    Error, EuropePmcId, Expression, ExpressionSyntax, FiniteNumber, GeneSymbol, HgncId, HgvsText,
    License, LocusGroup, Markdown, NcbiGeneId, NctId, PartialDate, Pmcid, Pmid, PopulationDataType,
    PopulationFrequency, PopulationSex, PredictorScore, RcvId, RsId, ScvId, SemanticScholarId,
    Statement, StudyType, Term, Text, TrialAgeBound, TrialArm, TrialContact, TrialDocument,
    TrialIntervention, TrialLocation, TrialOfficial, TrialOutcome, TrialPhase, TrialReference,
    TrialSex, TrialStatus, Variant, VariantId, VariantRef, VcvId, canonical_enum, text_value,
};

#[macro_use]
mod response_page;
mod paged;
pub use paged::Paged;
mod research;
pub use research::{
    ResearchEnvelope, ResearchFetch, ResearchHit, ResearchSearch, ResearchSearchMetadata,
};
mod predictor;
mod query;
// Export new public response types inside `response` and again at the crate root.
// `unreachable_pub` reports a missing crate-root re-export.
mod response;
pub use response::{SchemaCatalog, SchemaEntry};
pub use response::{
    SourceDateTime, SourceDateTimeValue, SourceLocalTimestampValue, SourceTimestamp,
    SourceTimestampTimezone, SourceTimestampValue, SourceUpdated,
};
mod source;

pub use predictor::{
    ComponentLabel, ComponentTerms, ComponentUpload, CosmicCoordinates, CosmicIdentity,
    CosmicRecord, PredictorContext, SubjectQualification, VariantComponent, VariantConservation,
    VariantCosmic, VariantPredictors,
};
pub use query::PhaseFilter;
pub use query::SponsorType;
pub use query::TrialDocumentGet;
pub use query::VariantPopulationQuery;
pub use query::{ArticleEdges, EdgeSource};
pub use query::{ArticleEvidence, ArticlePdf, ArticleRecommend};
pub use query::{
    ArticleFilter, ArticleSearch, ArticleSort, ArticleSource, ArticleType, RetractionFilter,
};
pub use query::{ArticleGet, ArticleSection, ArticleSectionWord};
pub use query::{ArticleSupplement, ArticleText, SheetRows};
pub use query::{COMMANDS, CommandSpec, Rewrite, command_paths};
pub use query::{CivicClaimKind, VariantSection};
pub use query::{GeneGet, GeneSearch, GeneSection, GeneSectionWord, Request};
pub use query::{
    MyVariantQuery, VariantRegion, VariantSearch, VariantSearchField, VariantSearchSource,
    protein_query_forms,
};
pub use query::{
    NciBiomarkerUse, NciStatusFilter, TrialFilter, TrialPhaseFilter, TrialSearch, TrialSource,
    TrialStatusFilter,
};
pub use query::{NormalizationSource, VariantGet, VariantNormalize, VariantResolver};
pub use query::{RecurrenceSource, VariantRecurrenceQuery};
pub use query::{TrialGet, TrialSection};
pub use query::{VariantInput, VariantNamespace};
pub use response::TrialParameter;
pub use response::{AnnotatedEntity, AnnotationKind, Annotations, ArticleSections, Tldr};
pub use response::{ArticleCardItem, ArticleCards, ArticleMatch, ArticleResolution};
pub use response::{ArticleEdgeItem, ArticleEdgePage, CitationIntent, Citations};
pub use response::{
    ArticleEvidenceResult, ArticleRecommendations, CitationContexts, CitationParagraphs,
    EvidenceContext, EvidenceNotice, EvidenceParagraph,
};
pub use response::{
    ArticleItem, ArticleKey, ArticlePage, ArticleSentence, CacheProblem, LitVarCandidate, PoolInfo,
    SourceNotice,
};
pub use response::{ArticlePdfResult, OpenAccessStatus, PdfLink, PdfSaved};
pub use response::{
    ArticleSaved, ArticleTextResult, FigureCaption, FullText, TextContent, TextHit, TextPart,
    TextPartKind,
};
pub use response::{
    ArticleSupplementResult, CellFragment, FileKind, FirstSheetRow, SheetInfo, SheetRow,
    SupplementContent, SupplementFile, Supplements,
};
pub use response::{BundleSize, ComponentHealth, StoreSize};
pub use response::{
    CancerPtpm, Capped, CivicGene, ClinGenGene, ClinGenValidity, GenccGene, GenccSubmission,
    GeneExpression, PathwayRef, ReactomeId, RnaAbundance, RnaUnit, TissueNtpm,
};
pub use response::{
    CivicClaimPage, CivicCondition, CivicMember, CivicMembership, CivicNativeClaim, CivicProfile,
    CivicStatement, CivicTherapy, HotspotAlternate, HotspotMembership, HotspotRecord,
    OncokbCitation, OncokbConcept, OncokbVariantRecord, VariantCivic, VariantHotspots,
    VariantOncokb, VariantSections,
};
pub use response::{
    ConfigurationHealth, ConfigurationNotice, ConfigurationNoticeCode, ConfigurationVariable,
    DisableGroup, DisabledComponent, Health, HttpHealth, LocalBundleState, SourceHealth,
};
pub use response::{Constraint, GeneSections, Protein};
pub use response::{
    EmptyQuery, Envelope, GeneCards, GeneItem, GenePage, ItemStatus, PageInfo, ResponseBody,
    Section, SourceOutcome, SourceRecord, SourceState, Status, Step, StepKind,
};
pub use response::{ErrorCode, ErrorEnvelope, ErrorMessage, ErrorStatus, Problem, ProblemCode};
pub use response::{
    GnomadCounts, GwasAssociation, GwasEffectAllele, GwasTrait, VariantGwas, VariantPopulation,
    VariantPopulationResult,
};
pub use response::{
    Help, HelpArgument, HelpCommand, HelpQuery, HelpWord, KeyHelp, KeyState, KeyTier, Oncokb,
    OncokbCard, OncokbGeneType, OncokbTranscript, RegisteredKey, RegisteredKeys, StoreClean,
    SyncQuery, SyncResult,
};
pub use response::{
    NciArm, NciArmRecord, NciCriterion, NciEligibility, NciEligibilityPage,
    NciEligibilityStructured, NciIntervention, NciOutcome, NciTextPage, NciTrialSections,
};
pub use response::{
    NciDisease, NciDiseaseRef, NciMatchedSite, NciTrial, NciTrialBiomarker, NciTrialCards,
    NciTrialItem, NciTrialPage,
};
pub use response::{
    NormalizationField, NormalizationMessage, NormalizationResult, VariantNormalization,
};
pub use response::{RecurrenceGranularity, RecurrenceStudy, RecurrenceTerms, VariantRecurrence};
pub use response::{
    TrialArmRecord, TrialCardItem, TrialCards, TrialContactRecord, TrialEligibilityPage,
    TrialRecords, TrialSections,
};
pub use response::{TrialConditionResolution, TrialPage};
pub use response::{TrialDocumentResult, TrialDocumentSaved};
pub use response::{
    VariantCard, VariantGetQuery, VariantHeader, VariantResolution, VariantTranscript,
};
pub use response::{VariantPage, VariantSearchExclusionReason, VariantSearchExclusions};
pub use source::{Reason, Source, Use};

pub use query::{OrganizationSearch, OrganizationSource};

mod article;

pub use article::{
    AbstractPart, Article, ArticleInput, ArticleMatchKind, Correction, CorrectionKind,
    CorrectionRelation,
};
pub use article::{ArticleHeader, Indexing, MeshHeading, MeshQualifier, PublicationType};

pub use query::GeneInput;
pub use response::{GeneHeader, GeneResolution, Match};

pub use response::TrialHeader;

pub use query::{AssertionId, AssertionSource, VariantAssertionsGet};
pub use response::{
    AssertionAssessment, AssertionCondition, AssertionConditionRelationship, AssertionDetail,
    AssertionDocument, AssertionDomain, AssertionKind, UnmappedAssertion, VariantAssertion,
    VariantAssertions,
};

pub use query::{
    DiseaseGet, DiseaseId, DiseaseOntology, DiseaseSearch, DiseaseSection, DiseaseSource,
};

pub use response::{
    DiseaseAssociationItem, DiseaseCard, DiseaseConcept, DiseaseGenes, DiseaseHeader,
    DiseaseHierarchy, DiseaseIncidence, DiseasePage, DiseasePhenotypes, DiseaseSite, DiseaseSites,
    DiseaseSurvival, DiseaseSynonyms, DiseaseXrefs,
};
mod drug;
pub use drug::{
    DEFAULT_DRUG_APPROVALS_LIMIT, DrugApprovals, DrugCard, DrugCivic, DrugCivicClaim,
    DrugCivicMember, DrugCivicPage, DrugCivicProfile, DrugCivicStreams, DrugFdaCandidate,
    DrugFdaDocument, DrugFdaIngredient, DrugFdaPage, DrugFdaProduct, DrugFdaProperty,
    DrugFdaSubmission, DrugGet, DrugHeader, DrugIndications, DrugInteractions, DrugLabel,
    DrugLabelFragment, DrugLabelText, DrugNameMatch, DrugNameResolution, DrugNameSelector,
    DrugOncokbLookup, DrugOncokbMissing, DrugPage, DrugRecallCard, DrugRecallClassification,
    DrugRecallCountUnit, DrugRecallHeader, DrugRecallPage, DrugRecords, DrugRegulatory, DrugSearch,
    DrugSection, DrugSource, DrugTargets, DrugWhoCard, DrugWhoGet, DrugWhoPage, DrugWhoRow,
    DrugXrefs, OPENFDA_LABEL_HEADINGS,
};

pub use query::{CheckedQuery, QueryInvalid};

mod adverse_event;
pub use adverse_event::{
    AdverseEventDevice, AdverseEventDevices, AdverseEventDrug, AdverseEventDrugs,
    AdverseEventFaersCard, AdverseEventGet, AdverseEventHeader, AdverseEventIdentifierRecords,
    AdverseEventIdentifierSection, AdverseEventIdentifiers, AdverseEventIdentityCounts,
    AdverseEventMaudeCard, AdverseEventNarrative, AdverseEventNarrativeItem, AdverseEventPage,
    AdverseEventReaction, AdverseEventReactionMatch, AdverseEventReactions, AdverseEventRecords,
    AdverseEventSearch, AdverseEventSection, AdverseEventSerious, AdverseEventSource, ReportUnit,
    VAERS_INFLUENZA, VaersCell, VaersNotice, VaersRow, VaersScope, VaersSection, VaersSummary,
    vaers_vaccine_code,
};

mod pgx;
pub use pgx::*;
pub use query::{
    DiagnosticGet, DiagnosticGetSource, DiagnosticId, DiagnosticSearch, DiagnosticSearchSource,
    DiagnosticSection,
};
pub use response::{
    DiagnosticCandidate, DiagnosticCard, DiagnosticCompanion, DiagnosticConditions,
    DiagnosticGenes, DiagnosticLaboratory, DiagnosticMethods, DiagnosticPage, DiagnosticRegulatory,
    DiagnosticRegulatoryField, DiagnosticRegulatoryFragment, DiagnosticSupplements, GtrCard,
    GtrHeader, GtrNativeStatus,
};

pub use response::{
    DiagnosticClearance, DiagnosticPmaCard, DiagnosticPmaHeader, DiagnosticPmaRegulatory,
};

/// Source-specific diagnostic page counting vocabulary.
pub use response::DiagnosticCountUnit;

#[cfg(feature = "schema")]
pub use drug::DrugGetCommand;

mod gwas;
pub use gwas::*;
mod phenotype;
pub use phenotype::*;
pub use query::{
    PathwayEventsQuery, PathwayGenesQuery, PathwayGet, PathwaySearch, PathwaySection, PathwaySource,
};

pub use response::{
    PathwayCard, PathwayEvents, PathwayGenes, PathwayHeader, PathwayPage, PathwayPageInfo,
    PathwayTotalKind, WikiPathwaysCatalogReceipt, WikiPathwaysSync,
};

mod protein;
pub use protein::*;
mod study;
pub use study::*;

mod cell_line;
pub use cell_line::*;

mod author;
pub use author::*;

pub use response::{
    AlphaGenomeBiosample, AlphaGenomeDataType, AlphaGenomeGene, AlphaGenomeInterval,
    AlphaGenomeOntology, AlphaGenomeScorer, AlphaGenomeScorerPage, AlphaGenomeTrack,
    AlphaGenomeValue, AlphaGenomeVariant, VariantAlphaGenome,
};
pub use response::{
    AlphaGenomeBiosampleType, AlphaGenomeEndedness, AlphaGenomeOntologyType, AlphaGenomeStrand,
};
pub use response::{AlphaGenomeNonfiniteKind, AlphaGenomeScalar};

pub use response::{CivicKind, CivicSignificance, CivicStatus};

pub use response::{NciPhaseFilter, NciTrialPhase};

pub use response::{DeviceRecallCard, DeviceRecallHeader};

pub use response::{Organization, OrganizationCoordinates, OrganizationPage};

pub use query::{BiomarkerGet, BiomarkerSearch, BiomarkerSource};
pub use response::{BiomarkerCard, BiomarkerHeader, BiomarkerPage, BiomarkerResult};

pub use response::{UsageArguments, UsageError};

pub use response::{Reply, ReplyBody, ReplyData};

mod coverage;
pub use coverage::{command_implicit_sources, command_section_sources};

#[cfg(feature = "schema")]
pub use adverse_event::AdverseEventSearchCommand;
pub use response::{
    TrialAdverseEvents, TrialEvent, TrialEventGroup, TrialEventStats, TrialPostedCard,
    TrialPostedOutcome, TrialResultAnalysis, TrialResultCategory, TrialResultClass,
    TrialResultCount, TrialResultDenominator, TrialResultGroup, TrialResultMeasurement,
    TrialResultRecord,
};
