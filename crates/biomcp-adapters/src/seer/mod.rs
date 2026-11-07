//! Canonical seer publisher declaration.
use crate::mapped::problem;
use biomcp_fetch::Input;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::{Source, Use};
use std::time::Duration;
mod map;
mod raw;
mod selectors;
use biodata::{DiseaseStatistic, Text};
use biomcp_fetch::{Arg, Endpoint, Freshness, Leg, ReplyMetadata, RequestError};
use biomcp_types::DiseaseSite;
pub use selectors::{Selector, SelectorError};
/// Resolve source selectors once before any network access.
pub fn selection(query: &biomcp_types::DiseaseGet, id: &Text) -> Result<Selection, SelectorError> {
    let period = if query.section() == Some(biomcp_types::DiseaseSection::Survival) {
        (Selector::Interval, query.interval())
    } else {
        (Selector::Period, query.period())
    };
    let selectors = [
        (Selector::Dataset, query.dataset()),
        period,
        (Selector::Sex, query.sex()),
        (Selector::Race, query.race()),
        (Selector::AgeRange, query.age_range()),
    ]
    .into_iter()
    .filter_map(|(selector, value)| selector.resolve(value).err().map(|_| selector))
    .collect::<Vec<_>>();
    if !selectors.is_empty() {
        return Err(SelectorError { selectors });
    }
    let statistic = if query.section() == Some(biomcp_types::DiseaseSection::Survival) {
        Statistic::Survival {
            interval: Selector::Interval.resolve(query.interval())?,
        }
    } else {
        Statistic::Incidence {
            period: Selector::Period.resolve(query.period())?,
        }
    };
    Ok(Selection {
        site: id.clone(),
        sex: Selector::Sex.resolve(query.sex())?,
        race: Selector::Race.resolve(query.race())?,
        age_range: Selector::AgeRange.resolve(query.age_range())?,
        dataset: Selector::Dataset.resolve(query.dataset())?,
        statistic,
    })
}
/// A selected native statistic, with its own applicable selector.
#[derive(Debug)]
pub enum Statistic {
    /// Recent stage-stratified incidence.
    Incidence {
        /// Native year-range code.
        period: Text,
    },
    /// Annual relative survival; the source's annual period is fixed separately.
    Survival {
        /// Native relative-survival interval.
        interval: Text,
    },
}
/// Explicit source selectors. None establishes returned dataset membership.
#[derive(Debug)]
pub struct Selection {
    /// Caller-selected catalog code.
    pub site: Text,
    /// Native sex selector.
    pub sex: Text,
    /// Native race selector.
    pub race: Text,
    /// Native age selector.
    pub age_range: Text,
    /// Requested data-model selector.
    pub dataset: Text,
    /// Independent statistic-specific selector.
    pub statistic: Statistic,
}
/// A locally paged checked source response.
#[derive(Debug)]
pub struct Page<T> {
    /// Accepted rows and native traversal facts.
    pub window: crate::NativeWindow<T>,
    /// Original response metadata, never the page-read time.
    pub metadata: ReplyMetadata,
    /// Source fields that remain unverified or contradict a request.
    pub problems: Vec<biomcp_types::Problem>,
    /// Publisher code labels for required statistic selectors; empty for statistic pages.
    pub selectors: Box<[biodata::StatisticSelector]>,
}
/// Publisher policy shared by health, admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Seer,
    base: "https://seer.cancer.gov/statistics-network/explorer/source/content_writers",
    terms: Terms {
        name: "SEER/NCI Using This Website",
        url: "https://seer.cancer.gov/about/using-website.html",
        usage: Use::Open,
    },
    quota: &crate::quotas::SEER,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};
const CATALOG: Endpoint = SPEC.get("/get_var_formats.php").fresh(Freshness::Days(1));
const SERIES: Endpoint = SPEC.get("/render_region_5.php").fresh(Freshness::Days(1));
/// Read the native catalog without selecting an ontology mapping.
pub fn catalog(leg: &Leg<'_>, limit: u32, offset: u32) -> Result<Page<DiseaseSite>, RequestError> {
    catalog_named(leg, None, limit, offset)
}
/// Filter the full publisher catalog before local paging; no guessed identities.
pub fn catalog_named(
    leg: &Leg<'_>,
    text: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<Page<DiseaseSite>, RequestError> {
    let (mut raw, metadata) = leg
        .call(
            &CATALOG,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Catalog>::json(),
        )
        .map(|received| (received.value, received.meta))?;
    if let Some(text) = text {
        let text = text.trim().to_lowercase();
        raw.sites.retain(|site| {
            site.as_ref().is_none_or(|site| {
                raw.formats
                    .site
                    .as_ref()
                    .and_then(crate::raw::Field::as_ref)
                    .and_then(|labels| labels.get(&site.value.to_string()))
                    .and_then(crate::raw::Field::as_ref)
                    .is_none_or(|label| label.as_str().to_lowercase().contains(&text))
            })
        });
    }
    let total = raw.sites.len() as u64;
    let mut problems = crate::mapped::Problems {
        source: Source::Seer,
        record: None,
        list: vec![
            problem(Source::Seer, None, "catalog.publisher_date"),
            problem(Source::Seer, None, "catalog.publisher_version"),
        ],
    };
    let selectors =
        map::choices(&raw.formats).map_err(|_| biomcp_fetch::InputError::Serialization)?;
    let labels = problems.field("VariableFormats.site", raw.formats.site);
    let mut rows = Vec::new();
    let mut refusals = crate::Refusal::None;
    let window: Vec<_> = raw
        .sites
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    let received = window.len() as u32;
    for site in window {
        problems.record = None;
        if let Some(site) = problems.field("CancerSites.value", Some(site)) {
            let code = crate::mapped::text(&site.value.to_string())
                .map_err(|_| biomcp_fetch::InputError::Serialization)?;
            problems.record = Some(code.clone());
            let label = problems
                .required(
                    "VariableFormats.site",
                    labels
                        .as_ref()
                        .and_then(|v| v.get(&site.value.to_string()))
                        .and_then(crate::raw::Field::as_ref)
                        .cloned(),
                )
                .map(Into::into);
            let active = problems.required_field("CancerSites.active", site.active);
            rows.push(DiseaseSite {
                code,
                label,
                active,
            });
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    Ok(Page {
        window: crate::NativeWindow::new(rows, offset, limit, Some(total), received, refusals),
        metadata,
        problems: problems.list,
        selectors,
    })
}
impl Selection {
    fn params(&self) -> Vec<(&'static str, Arg<'_>)> {
        let survival = matches!(self.statistic, Statistic::Survival { .. });
        let mut params = vec![
            ("site", Arg::Text(self.site.as_str())),
            ("data_type", Arg::Num(if survival { 4 } else { 1 })),
            ("graph_type", Arg::Num(if survival { 1 } else { 10 })),
            ("compareBy", Arg::Text("sex")),
            ("sex", Arg::Text(self.sex.as_str())),
            ("race", Arg::Text(self.race.as_str())),
            ("age_range", Arg::Text(self.age_range.as_str())),
        ];
        match &self.statistic {
            Statistic::Incidence { period } => {
                params.push(("year_range", Arg::Text(period.as_str())))
            }
            Statistic::Survival { interval } => {
                params.push(("relative_survival_interval", Arg::Text(interval.as_str())));
                params.push(("year_range", Arg::Num(31)));
            }
        }
        params.push(("data_model", Arg::Text(self.dataset.as_str())));
        params
    }
}
/// Read one native response and page it locally without another provider request.
pub fn statistics(
    leg: &Leg<'_>,
    query: &Selection,
    limit: u32,
    offset: u32,
) -> Result<Page<DiseaseStatistic>, RequestError> {
    let (raw, metadata) = leg
        .call(
            &SERIES,
            Input::Params(&query.params()),
            biomcp_fetch::Read::<raw::Document>::json_string()
                .check(&|r| {
                    let expected = if matches!(query.statistic, Statistic::Survival { .. }) {
                        "rel_rate"
                    } else {
                        "rate"
                    };
                    let site_index = r.info.keys.iter().position(|key| key == "site");
                    let matching_site = site_index.is_some_and(|index| {
                        r.data
                            .iter()
                            .all(|(key, _)| key.split('_').nth(index) == Some(query.site.as_str()))
                    });
                    if r.info.measure == expected && matching_site {
                        Ok(())
                    } else {
                        Err(biomcp_fetch::BadReply)
                    }
                })
                .cache_if(&map::identities_admitted),
        )
        .map(|received| (received.value, received.meta))?;
    let labels = leg
        .call(
            &CATALOG,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Catalog>::json(),
        )?
        .value
        .formats;
    let (rows, problems) = map::rows(raw, query, &labels).map_err(|_| {
        RequestError::Source(biomcp_fetch::SourceError {
            source: Source::Seer,
            reason: biomcp_types::Reason::BadReply,
            status: Some(200),
            tls_cause: None,
            retry_after: None,
        })
    })?;
    Ok(Page {
        window: crate::NativeWindow::local(rows, offset, limit),
        metadata,
        problems,
        selectors: Box::default(),
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
