mod card;
mod indexing;
pub use indexing::{Indexing, MeshHeading, MeshQualifier};
mod input;
use crate::{Doi, EuropePmcId, PartialDate, Pmcid, Pmid, Text};
pub use card::{AbstractPart, Article, Correction, CorrectionKind, CorrectionRelation};
pub use input::{ArticleInput, ArticleMatchKind};
use serde::{Deserialize, Serialize};
biodata::canonical_enum! {
    /// Clinically meaningful PubMed publication types.
    PublicationType, SourceUnknown {
        /// Randomized controlled trial.
        RandomizedControlledTrial => "randomized-controlled-trial",
        /// Clinical trial.
        ClinicalTrial => "clinical-trial",
        /// Phase one trial.
        ClinicalTrialPhase1 => "clinical-trial-phase-1",
        /// Phase two trial.
        ClinicalTrialPhase2 => "clinical-trial-phase-2",
        /// Phase three trial.
        ClinicalTrialPhase3 => "clinical-trial-phase-3",
        /// Phase four trial.
        ClinicalTrialPhase4 => "clinical-trial-phase-4",
        /// Meta-analysis.
        MetaAnalysis => "meta-analysis",
        /// Systematic review.
        SystematicReview => "systematic-review",
        /// Review.
        Review => "review",
        /// Practice guideline.
        PracticeGuideline => "practice-guideline",
        /// Guideline.
        Guideline => "guideline",
        /// Case report.
        CaseReport => "case-report",
        /// Observational study.
        ObservationalStudy => "observational-study",
        /// Multicenter study.
        MulticenterStudy => "multicenter-study",
        /// Letter.
        Letter => "letter",
        /// Comment.
        Comment => "comment",
        /// Editorial.
        Editorial => "editorial",
        /// Preprint.
        Preprint => "preprint",
        /// Retracted publication.
        RetractedPublication => "retracted-publication",
        /// Retraction notice.
        RetractionNotice => "retraction-notice",
        /// Erratum.
        Erratum => "erratum",
    }
}
/// Citation metadata from PubMed or Europe PMC.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ArticleHeader {
    /// Checked PubMed identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Pmid"))]
    pub pmid: Option<Pmid>,
    /// Checked DOI, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<Doi>,
    /// Checked PMC identity, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<Pmcid>,
    /// Europe PMC source-local identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub europepmc_id: Option<EuropePmcId>,
    /// Whole source title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(with = "Text"))]
    pub title: Option<Text>,
    /// Journal name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<Text>,
    /// Publication date at source precision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<PartialDate>,
    /// Electronic publication date when different from the publication date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub electronic_date: Option<PartialDate>,
    /// Clinical publication types.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub types: Box<[PublicationType]>,
    /// The provider does not supply checked publication types.
    #[serde(default, skip_serializing_if = "is_false")]
    pub types_unknown: bool,
    /// First author in source order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_author: Option<Text>,
    /// Exact Semantic Scholar ID for the displayed native first author, when supplied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_author_id: Option<biodata::SemanticScholarAuthorId>,
    /// Whole source author count.
    pub author_count: u32,
}

fn is_false(value: &bool) -> bool {
    !value
}
