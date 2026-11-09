//! Resolve complete source names before the unchanged native-ID path.
use crate::{Ctx, Outcome, status};
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugGet, DrugHeader, DrugNameMatch, DrugNameResolution, DrugSearch, DrugSource, ReplyData,
    Step, StepKind, UsageArguments, UsageError,
};

type Rank = (u8, u8, String);
/// Most MyChem rows a name get reads while looking for a drug record with an exact name.
const MYCHEM_NAME_CAP: u32 = 250;

pub(super) fn get(cx: &Ctx<'_>, query: &DrugGet) -> Outcome {
    let deadline = cx.fetch.deadline(cx.budget);
    let name = query.input().to_string();
    let search = match DrugSearch::named(query.input().clone(), query.source(), 50) {
        Ok(search) => search,
        Err(_) => return status::empty(ReplyData::Usage(UsageError::arguments())),
    };
    let mut unsettled = false;
    let (mut discovery, mut candidates) = if query.source() == DrugSource::Drugsfda {
        fda(cx, query, &search)
    } else if query.source() == DrugSource::OpenfdaLabel {
        label(cx, query, &search)
    } else if query.source() == DrugSource::Mychem {
        let (out, candidates, stopped) = mychem(cx, query, &search);
        unsettled = stopped;
        (out, candidates)
    } else if query.source() == DrugSource::Chembl {
        chembl(cx, query, &search)
    } else if query.source() == DrugSource::Ema {
        ema(cx, query, &search)
    } else {
        discover(cx, &search)
    };
    if discovery.dropped
        || discovery
            .sources
            .iter()
            .any(|s| s.state != biomcp_types::SourceState::Ok)
    {
        status::item(&mut discovery, false, true, false);
        return discovery;
    }
    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    let mut seen = std::collections::BTreeSet::new();
    candidates.retain(|(_, header)| seen.insert(header.id.to_string()));
    let Some((rank, selected)) = candidates.first() else {
        discovery.body = ReplyData::Usage(UsageError::Arguments(UsageArguments {
            argument: Some("input".into()),
            next: vec![Step {
                kind: StepKind::Disambiguate,
                args: vec![
                    "search".into(),
                    "drug".into(),
                    name.into(),
                    "--source".into(),
                    query.source().as_str().into(),
                ]
                .into(),
            }],
            ..Default::default()
        }));
        return discovery;
    };
    let resolved = match query.with_native_id(selected.id.clone()) {
        Ok(resolved) => resolved,
        Err(_) => return super::rejected_input(query.source(), "input", discovery),
    };
    let out = super::selected_get(&within(cx, deadline.remaining()), &resolved);
    let mut out = chosen_name(query, rank.0, &candidates, discovery, out);
    if unsettled && out.status() == biomcp_types::Status::Complete {
        // A product-only answer, while later matches may hold the drug itself.
        status::item(&mut out, true, false, true);
    }
    out
}

fn chosen_name(
    query: &DrugGet,
    rank: u8,
    candidates: &[(Rank, DrugHeader)],
    discovery: Outcome,
    mut out: Outcome,
) -> Outcome {
    let ranking = (if query.source() == DrugSource::Drugsfda {
        "exact brand before generic; NDA/BLA before ANDA, then earliest approval"
    } else {
        "exact brand before generic; source search order among equals"
    })
    .into();
    let resolution = DrugNameResolution {
        input: query.input().clone(),
        basis: if rank == 0 {
            DrugNameMatch::Brand
        } else {
            DrugNameMatch::Generic
        },
        ranking,
        total: candidates.len() as u64,
        alternatives: candidates
            .iter()
            .skip(1)
            .take(5)
            .map(|(_, h)| Step {
                kind: StepKind::Disambiguate,
                args: vec![
                    "get".into(),
                    "drug".into(),
                    h.id.to_string().into(),
                    "--source".into(),
                    query.source().as_str().into(),
                ]
                .into(),
            })
            .collect(),
    };
    annotate(&mut out.body, resolution);
    out.problems.extend(discovery.problems);
    for source in discovery.sources {
        if let Some(existing) = out.sources.iter_mut().find(|s| s.source == source.source) {
            status::retain_source_state(existing, &source);
        } else {
            out.sources.push(source);
        }
    }
    out
}

fn discover(cx: &Ctx<'_>, search: &DrugSearch) -> (Outcome, Vec<(Rank, DrugHeader)>) {
    let deadline = cx.fetch.deadline(cx.budget);
    let mut query = search.clone();
    let mut out = super::search(&within(cx, deadline.remaining()), &query);
    let mut candidates = Vec::new();
    let mut incomplete = false;
    let mut problems = Vec::new();
    loop {
        incomplete |= out.dropped;
        problems.append(&mut out.problems);
        if let ReplyData::DrugPage {
            body: Some(page), ..
        } = &out.body
        {
            for header in &page.items {
                let rank = match header.matched_name {
                    Some(DrugNameMatch::Brand | DrugNameMatch::Both) => Some(0),
                    Some(DrugNameMatch::Generic) => Some(1),
                    _ if header.name.as_ref().is_some_and(|n| {
                        search.query().is_some_and(|q| {
                            biomcp_types::DrugNameMatch::generic_name(n.as_str(), q.as_str())
                        })
                    }) =>
                    {
                        Some(1)
                    }
                    _ => None,
                };
                if let Some(rank) = rank {
                    candidates.push(((rank, 0, String::new()), header.clone()));
                }
            }
        }
        let Some(next_query) = out.next.iter().find_map(|step| match &step.1 {
            crate::Action::Run {
                request: biomcp_types::Request::SearchDrug(next),
                ..
            } if step.0 == StepKind::Page => Some(next.clone()),
            _ => None,
        }) else {
            break;
        };
        query = next_query;
        let next = super::search(&within(cx, deadline.remaining()), &query);
        if next
            .sources
            .iter()
            .any(|s| s.state != biomcp_types::SourceState::Ok)
        {
            out = next;
            break;
        }
        out = next;
    }
    out.dropped |= incomplete;
    out.problems.extend(problems);
    (out, candidates)
}

/// Rows read through one page of a source's total, and whether the answer is among them.
#[derive(Clone, Copy)]
struct Read {
    rows: u32,
    total: u64,
    settled: bool,
}
struct NativePage {
    candidates: Vec<(Rank, DrugHeader)>,
    /// How far a reader that may stop early has read, and whether it found its answer.
    read: Option<Read>,
    next_offset: Option<u32>,
    problems: Vec<biomcp_types::Problem>,
    refusals: biomcp_adapters::Refusal,
}
fn native_names(
    cx: &Ctx<'_>,
    query: &DrugSearch,
    spec: &'static biomcp_fetch::SourceSpec,
    read: impl Fn(&biomcp_fetch::Leg<'_>, u32) -> Result<NativePage, biomcp_fetch::RequestError>,
) -> (Outcome, Vec<(Rank, DrugHeader)>) {
    let (out, candidates, _) = native_pages(cx, query, spec, read);
    (out, candidates)
}
/// Read pages until the reader stops, keeping how far a reader that may stop early got.
fn native_pages(
    cx: &Ctx<'_>,
    query: &DrugSearch,
    spec: &'static biomcp_fetch::SourceSpec,
    read: impl Fn(&biomcp_fetch::Leg<'_>, u32) -> Result<NativePage, biomcp_fetch::RequestError>,
) -> (Outcome, Vec<(Rank, DrugHeader)>, Option<Read>) {
    let mut out = status::empty(biomcp_types::DrugPage::into_reply(query.clone(), None));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    let mut candidates = Vec::new();
    let mut offset = 0;
    let mut error = None;
    let mut examined = None;
    loop {
        match read(&leg, offset) {
            Ok(page) => {
                examined = page.read.map(|read| Read {
                    settled: read.settled || examined.is_some_and(|seen: Read| seen.settled),
                    ..read
                });
                out.include_refusals(page.refusals);
                out.problems.extend(page.problems);
                candidates.extend(page.candidates);
                let Some(next) = page.next_offset else {
                    break;
                };
                if next <= offset {
                    error = Some(biomcp_fetch::InputError::Serialization.into());
                    break;
                }
                offset = next;
            }
            Err(failure) => {
                error = Some(failure);
                break;
            }
        }
    }
    out.local = error.and_then(status::local);
    let remaining = leg.remaining();
    out.sources
        .push(status::source(query.source().source(), leg.finish(), error));
    status::search(&mut out);
    command.finish(remaining);
    (out, candidates, examined)
}
fn fda(cx: &Ctx<'_>, get: &DrugGet, query: &DrugSearch) -> (Outcome, Vec<(Rank, DrugHeader)>) {
    native_names(
        cx,
        query,
        &biomcp_adapters::drugsfda::SPEC,
        |leg, offset| {
            let page = biomcp_adapters::drugsfda::search_names(leg, get.input(), 50, offset)?;
            let next = offset + page.records.len() as u32;
            Ok(NativePage {
                read: None,
                candidates: page
                    .records
                    .iter()
                    .filter_map(|row| {
                        row.name_rank(get.input().as_str())
                            .map(|(name, app, date)| {
                                (
                                    (name, app, date.unwrap_or_else(|| "9999".into())),
                                    row.header(),
                                )
                            })
                    })
                    .collect(),
                next_offset: (next > offset && u64::from(next) < page.total).then_some(next),
                problems: page.problems,
                refusals: page.refusals,
            })
        },
    )
}
fn label(cx: &Ctx<'_>, get: &DrugGet, query: &DrugSearch) -> (Outcome, Vec<(Rank, DrugHeader)>) {
    native_names(
        cx,
        query,
        &biomcp_adapters::openfda_label::SPEC,
        |leg, offset| {
            let page = biomcp_adapters::openfda_label::search_names(leg, get.input(), 50, offset)?;
            let next_offset = page.window.next_offset();
            let mut candidates = Vec::new();
            for row in &page.window.rows {
                if let Some(matched) = row.exact_name_match(get.input()) {
                    candidates.push((
                        (
                            u8::from(matched == DrugNameMatch::Generic),
                            0,
                            String::new(),
                        ),
                        row.header()?,
                    ));
                }
            }
            Ok(NativePage {
                read: None,
                candidates,
                next_offset,
                problems: page.problems,
                refusals: page.window.refusals(),
            })
        },
    )
}

/// Read MyChem's free-text pages until one holds a drug record of the exact name. Its ranking can
/// place that record past looser matches and product-only records, so a page without one is no
/// answer, but the read stops at a page cap. Also says whether it stopped before finding one.
fn mychem(
    cx: &Ctx<'_>,
    get: &DrugGet,
    query: &DrugSearch,
) -> (Outcome, Vec<(Rank, DrugHeader)>, bool) {
    let rows = super::search_names::NAME_ROWS;
    let (mut out, mut candidates, examined) =
        native_pages(cx, query, &biomcp_adapters::mychem::SPEC, |leg, offset| {
            let (page, received, drug) =
                biomcp_adapters::mychem::search_names(leg, get.input().as_str(), rows, offset)?;
            let next = offset + received;
            // The middle rank puts a record with drug rows of its own before a product-only one.
            let candidates: Vec<_> = page
                .value
                .items
                .iter()
                .zip(drug)
                .filter_map(|(header, drug)| {
                    header.matched_name.map(|matched| {
                        let rank = u8::from(matched == DrugNameMatch::Generic);
                        ((rank, u8::from(!drug), String::new()), header.clone())
                    })
                })
                .collect();
            let settled = candidates.iter().any(|((_, product, _), _)| *product == 0);
            let more = received > 0
                && page
                    .value
                    .total
                    .is_some_and(|total| u64::from(next) < total);
            Ok(NativePage {
                read: page.value.total.map(|total| Read {
                    rows: next,
                    total,
                    settled,
                }),
                next_offset: (!settled && more && next < MYCHEM_NAME_CAP).then_some(next),
                candidates,
                problems: page.problems,
                refusals: page.refusals,
            })
        });
    // A product-only record answers only when no record with drug rows matched.
    if candidates.iter().any(|((_, product, _), _)| *product == 0) {
        candidates.retain(|((_, product, _), _)| *product == 0);
    }
    let Some(Read { rows, total, .. }) =
        examined.filter(|read| !read.settled && u64::from(read.rows) < read.total)
    else {
        return (out, candidates, false);
    };
    out.dropped |= candidates.is_empty();
    if let Some(source) = out
        .sources
        .iter_mut()
        .find(|s| s.source == biomcp_types::Source::Mychem)
    {
        source
            .notices
            .push(biomcp_types::SourceNotice::RequestGuidance {
                data_date: None,
                message: format!(
                    "Read the first {rows} of {total} matches without a drug record of that \
                     exact name; later matches were not examined."
                )
                .into(),
                retry_after_seconds: None,
            });
    }
    (out, candidates, true)
}
fn chembl(cx: &Ctx<'_>, get: &DrugGet, query: &DrugSearch) -> (Outcome, Vec<(Rank, DrugHeader)>) {
    let deadline = cx.fetch.deadline(cx.budget);
    let mut all = Vec::new();
    let mut outcome: Option<Outcome> = None;
    for selector in [
        biomcp_types::DrugNameSelector::Preferred,
        biomcp_types::DrugNameSelector::Synonym,
    ] {
        let (mut out, candidates) = native_names(
            &within(cx, deadline.remaining()),
            query,
            &biomcp_adapters::chembl::SPEC,
            |leg, offset| {
                let (page, _) = biomcp_adapters::chembl::search(
                    leg,
                    get.input().as_str(),
                    selector,
                    50,
                    offset,
                )?;
                let next_offset = page.window.next_offset();
                let refusals = page.window.refusals();
                let mut problems = page.problems;
                let mut candidates = Vec::new();
                for row in &page.window.rows {
                    if let Some(matched) = row.name_match(get.input().as_str()) {
                        let header = biomcp_adapters::chembl::molecule_header(row)
                            .map_err(|_| biomcp_fetch::InputError::Serialization)?;
                        problems.extend(header.problems);
                        candidates.push((
                            (
                                u8::from(matched == DrugNameMatch::Generic),
                                0,
                                String::new(),
                            ),
                            header.value,
                        ));
                    }
                }
                Ok(NativePage {
                    read: None,
                    candidates,
                    next_offset,
                    problems,
                    refusals,
                })
            },
        );
        if out.dropped
            || out
                .sources
                .iter()
                .any(|s| s.state != biomcp_types::SourceState::Ok)
        {
            return (out, Vec::new());
        }
        all.extend(candidates);
        if let Some(previous) = &mut outcome {
            out.problems.append(&mut previous.problems);
        }
        outcome = Some(out);
    }
    (
        outcome.unwrap_or_else(|| {
            status::empty(biomcp_types::DrugPage::into_reply(query.clone(), None))
        }),
        all,
    )
}

fn ema(cx: &Ctx<'_>, get: &DrugGet, query: &DrugSearch) -> (Outcome, Vec<(Rank, DrugHeader)>) {
    native_names(cx, query, &biomcp_adapters::ema::SPEC, |leg, _| {
        let rows = biomcp_adapters::ema::search_names(leg, get.input().as_str())?;
        let candidates = rows
            .value
            .into_iter()
            .filter_map(|header| {
                header.matched_name.map(|matched| {
                    (
                        (
                            u8::from(matched == DrugNameMatch::Generic),
                            0,
                            String::new(),
                        ),
                        header,
                    )
                })
            })
            .collect();
        Ok(NativePage {
            read: None,
            candidates,
            next_offset: None,
            problems: rows.problems,
            refusals: rows.refusals,
        })
    })
}

fn annotate(body: &mut ReplyData, resolution: DrugNameResolution) {
    let header = match body {
        ReplyData::DrugCard {
            body: Some(body), ..
        } => &mut body.header,
        ReplyData::DrugApprovals {
            body: Some(body), ..
        } => &mut body.identity,
        ReplyData::DrugLabel {
            body: Some(body), ..
        } => &mut body.identity,
        ReplyData::DrugRegulatory {
            body: Some(body), ..
        } => &mut body.identity,
        ReplyData::DrugTargets {
            body: Some(body), ..
        } => &mut body.identity,
        ReplyData::DrugIndications {
            body: Some(body), ..
        } => &mut body.identity,
        ReplyData::DrugCivic {
            body: Some(body), ..
        } => &mut body.identity,
        ReplyData::DrugInteractions {
            body: Some(body), ..
        } => &mut body.identity,
        ReplyData::DrugXrefs {
            body: Some(body), ..
        } => &mut body.identity,
        _ => return,
    };
    header.name_resolution = Some(resolution);
}

pub(super) fn within<'a>(cx: &Ctx<'a>, budget: std::time::Duration) -> Ctx<'a> {
    Ctx {
        fetch: cx.fetch,
        // Drug source credentials remain in the borrowed Fetch; no drug source contacts NCBI.
        ncbi_email: None,
        budget,
        fresh: cx.fresh,
        drug_name_limit: cx.drug_name_limit,
    }
}
