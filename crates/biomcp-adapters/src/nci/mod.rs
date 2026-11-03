//! Explicit keyed NCI Clinical Trials Search access, without vocabulary translation.
use biomcp_fetch::{Input, Read};
pub mod biomarker;
mod biomarker_map;
mod biomarker_raw;
mod eligibility;
pub mod organization;
mod organization_map;
mod organization_raw;
mod sites;
mod trial_biomarker;
mod trial_map;
mod trial_raw;
use biodata::{NctId, Text};

use biomcp_fetch::{
    Arg, BadReply, Credential, Endpoint, Freshness, KeyPlace, KeyUse, Leg, Reply, RequestError,
    SecretName, SourceSpec,
};
use biomcp_types::{NciDisease, NciTrial, TrialSearch};
use serde::{Deserialize, Serialize};
use std::time::Duration;
/// NCI requires the user's own API key for every request, including health probes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: biomcp_types::Source::Nci,
    base: "https://clinicaltrialsapi.cancer.gov/api/v2",
    key: KeyUse::Required(Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::NciApiKey,
        place: KeyPlace::Header("X-API-KEY"),
    }),
    quota: &crate::quotas::NCI,
    deadline: Duration::from_secs(15),
    terms: biomcp_fetch::Terms {
        name: "NCI Clinical Trials Search publisher terms; user's API key required",
        url: "https://www.cancer.gov/syndication/api",
        usage: biomcp_types::Use::Open,
    },
};
/// Native trial search, also used for one bounded health request.
pub const PAGE_LIMIT: u32 = 50;
/// Compact card sample; the diseases section returns every original concept.
pub const CARD_DISEASES: usize = 3;
/// Card summary preview; complete text remains in the summary section.
pub const CARD_SUMMARY_CHARACTERS: usize = 500;
/// Native endpoint with shared transport policy.
pub const SEARCH: Endpoint = SPEC.get("/trials").fresh(Freshness::Days(1));
const DETAIL: Endpoint = SPEC.get("/trials").fresh(Freshness::Days(1));
/// Native page, preserving provider counts and order.
#[derive(Debug, Deserialize)]
#[serde(from = "trial_map::Page")]
pub struct Page {
    /// Actual refused native records in this collection.
    pub refusals: crate::Refusal,
    /// Publisher total, never a local count.
    pub total: u64,
    /// Checked publisher trial rows.
    pub data: Vec<Trial>,
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.data.len() > PAGE_LIMIT as usize || self.data.len() as u64 > self.total {
            return Err(BadReply);
        }
        self.data.iter().try_for_each(Reply::check)
    }
}
/// Source-native identity and card fields. Extra source fields remain adapter-owned.
#[derive(Debug, Deserialize)]
#[serde(from = "trial_raw::Trial")]
pub struct Trial {
    /// Actual refused native child records.
    pub refusals: crate::Refusal,
    nci_id: Text,
    nct_id: NctId,
    brief_title: Option<Text>,
    current_trial_status: Option<Text>,
    phase: Option<biomcp_types::NciTrialPhase>,
    brief_summary: Option<Box<str>>,
    diseases: Vec<NciDisease>,
    biomarkers: Option<crate::raw::Field<Vec<crate::raw::Field<trial_biomarker::Biomarker>>>>,
    sites: Option<crate::raw::Field<Vec<crate::raw::Field<sites::Site>>>>,
    eligibility: Option<biomcp_types::NciEligibility>,
    arms: Box<[biomcp_types::NciArmRecord]>,
    outcome_measures: Box<[biomcp_types::NciOutcome]>,
    problems: Vec<biomcp_types::Problem>,
}
/// Native sections and compact card from one validated source record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Detail {
    /// Actual refused native child records persisted with the snapshot.
    #[serde(
        default = "crate::mapped::refusal_serde::none",
        with = "crate::mapped::refusal_serde"
    )]
    pub refusals: crate::Refusal,
    /// Rejected optional source fields.
    pub problems: Vec<biomcp_types::Problem>,
    /// Compact source-native card.
    pub card: NciTrial,
    /// Complete source summary, without trimming or rewriting.
    pub summary: Option<Box<str>>,
    /// Original disease concepts, without deduplication or inferred ranking.
    pub diseases: Box<[NciDisease]>,
    /// Original criteria and native structured fields.
    pub eligibility: Option<biomcp_types::NciEligibility>,
    /// Original outcomes in source order.
    pub outcomes: Box<[biomcp_types::NciOutcome]>,
    /// Original arm and intervention records with preserved relationships.
    pub arms: Box<[biomcp_types::NciArmRecord]>,
}
impl Reply for Trial {
    fn check(&self) -> Result<(), BadReply> {
        if !self.nci_id.as_str().starts_with("NCI-") {
            return Err(BadReply);
        }
        Ok(())
    }
}
impl Trial {
    /// Rejected native fields from this record.
    pub fn problems(&self) -> &[biomcp_types::Problem] {
        &self.problems
    }
    /// Native publisher card, with no clinical interpretation or inferred fields.
    pub fn card(&self, summary: bool) -> NciTrial {
        NciTrial {
            nci_id: self.nci_id.clone(),
            nct_id: self.nct_id,
            brief_title: self.brief_title.clone(),
            current_trial_status: self.current_trial_status.clone(),
            phase: self.phase.clone(),
            matched_biomarkers: Box::default(),
            matched_site: None,
            matching_sites: None,
            brief_summary: if summary {
                self.brief_summary
                    .as_ref()
                    .map(|s| biomcp_types::NciTextPage {
                        continuation: None,
                        text: s.chars().take(CARD_SUMMARY_CHARACTERS).collect(),
                        offset: 0,
                        total_characters: s.chars().count() as u32,
                    })
            } else {
                None
            },
            diseases: biomcp_types::TrialRecords {
                continuation: None,
                offset: 0,
                total: self.diseases.len() as u32,
                items: self
                    .diseases
                    .iter()
                    .take(CARD_DISEASES)
                    .map(|d| biomcp_types::NciDiseaseRef {
                        name: d.name.clone(),
                        nci_thesaurus_concept_id: d.nci_thesaurus_concept_id.clone(),
                    })
                    .collect(),
            },
        }
    }
    fn detail(self) -> Detail {
        let card = self.card(true);
        Detail {
            refusals: self.refusals,
            problems: self.problems,
            card,
            summary: self.brief_summary,
            diseases: self.diseases.into(),
            eligibility: self.eligibility,
            outcomes: self.outcome_measures,
            arms: self.arms,
        }
    }
}
/// Exact native parameters. Repeated includes remain separate parameters.
pub fn parameters(query: &TrialSearch) -> Vec<(String, String)> {
    let mut params = vec![
        ("size".into(), query.limit().to_string()),
        (
            "from".into(),
            query.next_page().map_or("0", Text::as_str).into(),
        ),
    ];
    for value in query.condition() {
        params.push((
            "diseases.nci_thesaurus_concept_id".into(),
            value.as_str().into(),
        ));
    }
    for country in query.country() {
        params.push(("sites.org_country".into(), country.to_string()));
    }
    if let Some(value) = query.keyword().first() {
        params.push(("keyword".into(), value.as_str().into()));
    }
    if let Some(name) = query.facility() {
        params.push(("sites.org_name".into(), name.as_str().into()));
    }
    if let Some(code) = query.biomarker_code() {
        params.push((
            "biomarkers.nci_thesaurus_concept_id".into(),
            code.to_string(),
        ));
    }
    if let Some(criterion) = query.biomarker_use() {
        params.push((
            "biomarkers.eligibility_criterion".into(),
            criterion.as_str().to_owned(),
        ));
    }
    if let Some(phase) = query.nci_phase() {
        params.push(("phase".into(), phase.as_str().into()));
    }
    if let Some(status) = query.nci_status() {
        params.push(("current_trial_status".into(), status.as_str().into()));
    }
    for field in [
        "nci_id",
        "nct_id",
        "brief_title",
        "current_trial_status",
        "phase",
        "diseases",
    ] {
        params.push(("include".into(), field.into()));
    }
    if query.biomarker_code().is_some() {
        params.push(("include".into(), "biomarkers".into()));
    }
    if query.facility().is_some() {
        params.push(("include".into(), "sites".into()));
    }
    params
}
/// Retrieve and check one source-native search page before cache admission.
pub fn search(leg: &Leg<'_>, params: &[(String, String)]) -> Result<Page, RequestError> {
    let args: Vec<_> = params
        .iter()
        .map(|(k, v)| (k.as_str(), Arg::Text(v)))
        .collect();
    let limit = params
        .iter()
        .find_map(|(name, value)| {
            (name == "size")
                .then(|| value.parse::<usize>().ok())
                .flatten()
        })
        .unwrap_or(PAGE_LIMIT as usize);
    leg.call(
        &SEARCH,
        Input::Params(&args),
        Read::<Page>::json().check(&|page| {
            if page.data.len() <= limit {
                Ok(())
            } else {
                Err(BadReply)
            }
        }),
    )
    .map(|received| (received.value, received.meta))
    .map(|(page, _)| page)
}
/// Exact identity check runs before cache admission and on replay or cache hits.
pub fn get(
    leg: &Leg<'_>,
    id: NctId,
) -> Result<(Detail, biomcp_fetch::ReplyMetadata), RequestError> {
    let endpoint = DETAIL.clone().with_path(format!("/trials/{id}"))?;
    let (raw, metadata) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            Read::<Trial>::json().check(&|r| {
                if r.nct_id == id {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok((raw.detail(), metadata))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
