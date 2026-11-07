use super::{Selection, Statistic, raw};
use crate::mapped::problem;
use biodata::{Attribution, DiseaseStatistic, FiniteNumber, StatisticEstimate, StatisticSelector};
use biomcp_fetch::BadReply;
use biomcp_types::{Problem, Source};

fn selector(name: &str, code: &str) -> Result<StatisticSelector, BadReply> {
    Ok(StatisticSelector {
        name: name.parse().map_err(|_| BadReply)?,
        code: code.parse().map_err(|_| BadReply)?,
        label: None,
    })
}
fn requested(query: &Selection) -> Result<Box<[StatisticSelector]>, BadReply> {
    let mut selectors = vec![
        selector("sex", query.sex.as_str())?,
        selector("race", query.race.as_str())?,
        selector("age_range", query.age_range.as_str())?,
        selector("data_model", query.dataset.as_str())?,
    ];
    match &query.statistic {
        Statistic::Incidence { period } => selectors.push(selector("year_range", period.as_str())?),
        Statistic::Survival { interval } => {
            selectors.push(selector("relative_survival_interval", interval.as_str())?);
            selectors.push(selector("year_range", "31")?);
        }
    }
    Ok(selectors.into())
}
fn attribution(key: &str) -> Result<Attribution, BadReply> {
    Ok(Attribution {
        approved: None,
        published: None,
        source: "seer".parse().map_err(|_| BadReply)?,
        record: Some(key.parse().map_err(|_| BadReply)?),
        version: None,
        release: None,
        submitter: None,
        evaluated: None,
        url: Some(
            format!("{}/render_region_5.php", super::SPEC.base)
                .parse()
                .map_err(|_| BadReply)?,
        ),
    })
}
fn cell<'a>(
    fields: &[String],
    row: &'a [Option<crate::raw::Field<raw::Cell>>],
    name: &str,
) -> Option<&'a crate::raw::Field<raw::Cell>> {
    fields
        .iter()
        .position(|f| f == name)
        .and_then(|i| row.get(i))
        .and_then(Option::as_ref)
}
fn number(
    fields: &[String],
    row: &[Option<crate::raw::Field<raw::Cell>>],
    name: &str,
    problems: &mut crate::mapped::Problems,
) -> Result<Option<biodata::WithUnit<FiniteNumber>>, BadReply> {
    let unit = match name {
        "rate" | "rate_se" | "rate_lower_ci" | "rate_upper_ci" => "per 100,000 people",
        _ => "%",
    }
    .parse()
    .map_err(|_| BadReply)?;
    Ok(problems.optional(
        name,
        cell(fields, row, name).map(|cell| {
            Ok::<_, BadReply>(biodata::WithUnit {
                value: cell.as_ref().ok_or(BadReply)?.number()?,
                unit: Some(unit),
            })
        }),
    ))
}
fn observed(
    fields: &[String],
    row: &[Option<crate::raw::Field<raw::Cell>>],
    measure: &str,
    problems: &mut crate::mapped::Problems,
) -> Result<Option<StatisticEstimate>, BadReply> {
    let estimate = StatisticEstimate {
        value: number(fields, row, measure, problems)?,
        standard_error: number(fields, row, &format!("{measure}_se"), problems)?,
        lower_ci: number(fields, row, &format!("{measure}_lower_ci"), problems)?,
        upper_ci: number(fields, row, &format!("{measure}_upper_ci"), problems)?,
        count: problems.optional(
            "count",
            cell(fields, row, "count").map(|v| v.as_ref().ok_or(BadReply)?.integer()),
        ),
    };
    Ok(
        if estimate.value.is_none()
            && estimate.standard_error.is_none()
            && estimate.lower_ci.is_none()
            && estimate.upper_ci.is_none()
            && estimate.count.is_none()
        {
            None
        } else {
            Some(estimate)
        },
    )
}
pub(super) fn rows(
    document: raw::Document,
    query: &Selection,
    labels: &raw::Formats,
) -> Result<(Vec<Option<DiseaseStatistic>>, Vec<Problem>), BadReply> {
    let mut requested = requested(query)?;
    label_selectors(&mut requested, labels);
    let mut problems = vec![
        problem(Source::Seer, None, "dataset_membership"),
        problem(Source::Seer, None, "registry_membership"),
    ];
    problems.extend(if matches!(query.statistic, Statistic::Incidence { .. }) {
        vec![
            problem(Source::Seer, None, "standard_population"),
            problem(Source::Seer, None, "observed_or_delay_adjusted_rate"),
        ]
    } else {
        vec![
            problem(Source::Seer, None, "modeling_method"),
            problem(Source::Seer, None, "eligibility"),
            problem(Source::Seer, None, "age_adjustment"),
        ]
    });
    let mut rows = Vec::new();
    for (key, series) in document.data {
        let mut returned = document
            .info
            .keys
            .iter()
            .zip(key.split('_'))
            .map(|(name, code)| selector(name, code))
            .collect::<Result<Vec<_>, _>>()?;
        returned.push(selector("year_range", &series.year_range.to_string())?);
        label_selectors(&mut returned, labels);
        mismatches(&requested, &returned, &mut problems);
        for row in series.data_series {
            let mut fields = crate::mapped::Problems {
                source: Source::Seer,
                record: crate::mapped::record(&key),
                list: Vec::new(),
            };
            let modeled = number(&document.info.fields, &row, "modeled_rel_rate", &mut fields)?
                .map(|value| StatisticEstimate {
                    value: Some(value),
                    standard_error: None,
                    lower_ci: None,
                    upper_ci: None,
                    count: None,
                });
            let year = if document.info.fields.iter().any(|v| v == "year") {
                let year = fields.required("year", cell(&document.info.fields, &row, "year"));
                let year =
                    fields.optional("year", year.map(|v| v.as_ref().ok_or(BadReply)?.year()));
                if year.is_none() {
                    rows.push(None);
                    problems.extend(fields.list);
                    continue;
                }
                year
            } else {
                None
            };
            rows.push(Some(DiseaseStatistic {
                requested_site: query.site.clone(),
                requested_selectors: requested.clone(),
                returned_selectors: returned.clone().into(),
                year,
                measure: document.info.measure.parse().map_err(|_| BadReply)?,
                observed: observed(
                    &document.info.fields,
                    &row,
                    &document.info.measure,
                    &mut fields,
                )?,
                modeled,
                attribution: attribution(&key)?,
            }));
            problems.extend(fields.list);
        }
    }
    Ok((rows, problems))
}
fn mismatches(
    requested: &[StatisticSelector],
    returned: &[StatisticSelector],
    problems: &mut Vec<Problem>,
) {
    for selector in returned {
        // Site identity has already passed the transport's checked cache admission.
        let expected = requested
            .iter()
            .find(|r| r.name == selector.name)
            .map(|r| &r.code);
        if expected.is_some_and(|e| e != &selector.code) {
            let field = format!("returned_selectors.{}", selector.name);
            if !problems.iter().any(|p| p.field.as_ref() == field) {
                problems.push(problem(Source::Seer, None, &field));
            }
        }
    }
}

pub(super) fn identities_admitted(document: &raw::Document) -> bool {
    let mut received = 0;
    let mut accepted = 0;
    let mut refusals = crate::Refusal::None;
    for row in document
        .data
        .iter()
        .flat_map(|(_, series)| &series.data_series)
    {
        received += 1;
        if !document.info.fields.iter().any(|field| field == "year")
            || cell(&document.info.fields, row, "year")
                .and_then(crate::raw::Field::as_ref)
                .is_some_and(|value| value.year().is_ok())
        {
            accepted += 1;
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    refusals.identities_admitted(accepted, received)
}

fn label_selectors(selectors: &mut [StatisticSelector], formats: &raw::Formats) {
    for selector in selectors {
        let labels = if selector.name.as_str() == "site" {
            formats.site.as_ref()
        } else {
            formats.labels.get(selector.name.as_str())
        };
        selector.label = labels
            .and_then(crate::raw::Field::as_ref)
            .and_then(|labels| labels.get(selector.code.as_str()))
            .and_then(crate::raw::Field::as_ref)
            .cloned()
            .map(Into::into);
    }
}

pub(super) fn choices(formats: &raw::Formats) -> Result<Box<[StatisticSelector]>, BadReply> {
    let mut rows = Vec::new();
    for selector in [
        super::Selector::Dataset,
        super::Selector::Period,
        super::Selector::Interval,
    ] {
        let name = match selector {
            super::Selector::Dataset => "data_model",
            super::Selector::Period => "year_range",
            _ => "relative_survival_interval",
        };
        for code in selector.valid_codes() {
            let mut row = self::selector(name, code)?;
            label_selectors(std::slice::from_mut(&mut row), formats);
            if row.label.is_some() {
                rows.push(row);
            }
        }
    }
    Ok(rows.into())
}
