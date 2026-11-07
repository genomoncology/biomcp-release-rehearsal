//! Resolve one typed drug name to the names its sources state, for trial and article search.
use crate::{Ctx, Outcome, status};
use biodata::Text;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    DrugGet, DrugNameMatch, DrugSource, Reason, ReplyData, SourceNotice, SourceOutcome,
    SourceState, SourceStatus,
};

/// Source rows read for one name; exact names rank first.
const NAME_ROWS: u32 = 50;
/// Lookups may spend one part in this many of what is left, so the search always keeps the rest.
const LOOKUP_SHARE: u32 = 3;
/// Most names sent to a provider that takes one request per name, under its request pacing.
const NAME_REQUESTS: usize = 4;
/// Most official names one search keeps past the name limit, so one OR group stays bounded.
const OFFICIAL_NAME_CEILING: usize = 10;

/// What one lookup settled about a typed name.
#[derive(Debug, Default)]
pub(crate) struct Resolved {
    /// Names to search, most wanted first; one name means the term stays as typed.
    pub names: Vec<Text>,
    /// A lookup failed and none gave names, so the search may be missing names.
    pub failed: bool,
}

enum Found {
    Names {
        names: Vec<Text>,
        preferred: Vec<String>,
        official: Vec<String>,
        record: String,
        evidence: Vec<SourceOutcome>,
        problems: Vec<biomcp_types::Problem>,
    },
    Ambiguous(Vec<SourceOutcome>),
    Failed(Vec<SourceOutcome>),
    Nothing,
}

/// Run lookups under a bounded share of the command, charging only what they spent.
pub(crate) fn bounded<'a, T>(
    deadline: &mut Deadline<'a>,
    run: impl FnOnce(&mut Deadline<'a>) -> T,
) -> T {
    let mut lookup = deadline.child(deadline.remaining() / LOOKUP_SHARE);
    let started = lookup.remaining();
    let value = run(&mut lookup);
    *deadline = deadline.after_stage(started.saturating_sub(lookup.remaining()));
    value
}

/// Walk the sources until one states the drug's names. A source with several structures, or
/// nothing more to give, passes to the next; a failed source is never read as an empty one.
/// ChEMBL comes before the label because a molecule record carries more names than one product.
/// `fits` says which names the searching provider can take.
pub(crate) fn resolve<'a>(
    cx: &Ctx<'a>,
    input: &Text,
    out: &mut Outcome,
    deadline: &mut Deadline<'a>,
    fits: fn(&str) -> bool,
) -> Resolved {
    let mut ambiguous = None;
    let mut failed = Vec::new();
    for source in [
        DrugSource::Mychem,
        DrugSource::Chembl,
        DrugSource::OpenfdaLabel,
    ] {
        if cx.fetch.disabled(source.source()) {
            continue;
        }
        if deadline.remaining().is_zero() {
            failed.push(biomcp_adapters::registry::outcome(
                source.source(),
                SourceStatus::Failed(Some(Reason::DeadlineExceeded)),
            ));
            continue;
        }
        let stage = cx.fetch.deadline(deadline.remaining());
        let started = stage.remaining();
        let lookup = super::resolve::within(cx, deadline.remaining());
        let found = if source == DrugSource::Mychem {
            mychem(&lookup, input)
        } else {
            card(&lookup, source, input, out)
        };
        *deadline = deadline.after_stage(started.saturating_sub(stage.remaining()));
        match found {
            Found::Names {
                names,
                preferred,
                official,
                record,
                evidence,
                problems,
            } => {
                let ranked = (preferred.as_slice(), official.as_slice());
                let kept = capped(names, ranked, input, cx.drug_name_limit, fits);
                let message = kept.message(source.source(), &record);
                let mut listed = kept.names.clone();
                listed.sort_by_key(lowered);
                record_evidence(out, evidence, input, &listed, message);
                out.problems.extend(problems);
                return Resolved {
                    names: kept.names,
                    failed: false,
                };
            }
            Found::Ambiguous(evidence) => ambiguous = ambiguous.or(Some(evidence)),
            Found::Failed(evidence) => failed.extend(evidence),
            Found::Nothing => {}
        }
    }
    let typed = vec![input.clone()];
    if !failed.is_empty() {
        let message = "Drug-name lookups did not finish, so names were not expanded; \
                       searched the typed term."
            .to_owned();
        record_evidence(out, failed, input, &typed, message);
        return Resolved {
            names: Vec::new(),
            failed: true,
        };
    }
    if let Some(evidence) = ambiguous {
        let message = "The name matches more than one drug structure; \
                       searched the typed term without expansion."
            .to_owned();
        record_evidence(out, evidence, input, &typed, message);
        return Resolved {
            names: typed,
            failed: false,
        };
    }
    Resolved::default()
}

/// Read the rows stating the name. Product rows and stereo forms are not separate drugs, and
/// product rows give no names when the drug has rows of its own.
fn mychem(cx: &Ctx<'_>, input: &Text) -> Found {
    let command = cx.fetch.command_deadline(cx.budget);
    let leg = cx
        .fetch
        .leg(&biomcp_adapters::mychem::SPEC, command.deadline())
        .fresh(cx.fresh);
    let result = biomcp_adapters::mychem::name_records(&leg, input.as_str(), NAME_ROWS);
    let error = result.as_ref().err().copied();
    let remaining = leg.remaining();
    let parent = status::source(DrugSource::Mychem.source(), leg.finish(), error);
    command.finish(remaining);
    let mapped = match result {
        Ok(mapped) => mapped,
        Err(_) => return failure(error, vec![parent]),
    };
    let carried = super::components(cx, &parent, false);
    let evidence: Vec<_> = std::iter::once(parent).chain(carried).collect();
    let structures: std::collections::BTreeSet<_> = mapped
        .value
        .iter()
        .filter_map(|row| row.structure.as_deref())
        .collect();
    if structures.len() > 1 {
        return Found::Ambiguous(evidence);
    }
    let Some(record) = mapped.value.iter().find(|row| row.structure.is_some()) else {
        return Found::Nothing;
    };
    let record = record.id.clone();
    // A product row with no structure of its own never describes this drug.
    let drug_rows = || mapped.value.iter().filter(|row| row.structure.is_some());
    let names = drug_rows()
        .flat_map(|row| row.names.iter().cloned())
        .collect();
    let preferred = drug_rows()
        .flat_map(|row| row.preferred.iter().cloned())
        .collect();
    let official = drug_rows()
        .flat_map(|row| row.official.iter().cloned())
        .collect();
    with_other_names(
        (names, preferred, official),
        input,
        record,
        evidence,
        mapped.problems,
    )
}

/// Read one exact source record through the get drug path.
fn card(cx: &Ctx<'_>, source: DrugSource, input: &Text, out: &mut Outcome) -> Found {
    let Some(query) = crate::next::checked(
        DrugGet::named(input.clone(), source),
        source.source(),
        &mut out.problems,
    ) else {
        return Found::Nothing;
    };
    let found = super::resolve::get(cx, &query);
    if found
        .sources
        .iter()
        .any(|s| s.source == source.source() && s.state != SourceState::Ok)
    {
        let error = found.local;
        return failure(error, found.sources);
    }
    if found.dropped {
        return Found::Nothing;
    }
    let ReplyData::DrugCard {
        body: Some(card), ..
    } = &found.body
    else {
        return Found::Nothing;
    };
    let names = super::aliases::names(card);
    if !names
        .iter()
        .any(|name| DrugNameMatch::same_identity(name.as_str(), input.as_str()))
    {
        return Found::Nothing;
    }
    // A ChEMBL record is one molecule; label rows are products of one drug and do not count.
    if source == DrugSource::Chembl && super::aliases::ambiguous(card) {
        return Found::Ambiguous(found.sources);
    }
    let preferred = names
        .first()
        .map(|name| name.to_string())
        .into_iter()
        .collect();
    let names = names.into_iter().cloned().collect();
    let official = card.official_names.iter().map(Text::to_string).collect();
    let record = card.header.id.to_string();
    with_other_names(
        (names, preferred, official),
        input,
        record,
        found.sources,
        found.problems,
    )
}

fn with_other_names(
    (names, preferred, official): (Vec<Text>, Vec<String>, Vec<String>),
    input: &Text,
    record: String,
    evidence: Vec<SourceOutcome>,
    problems: Vec<biomcp_types::Problem>,
) -> Found {
    if names
        .iter()
        .all(|name| DrugNameMatch::same_name(name.as_str(), input.as_str()))
    {
        return Found::Nothing;
    }
    Found::Names {
        names,
        preferred,
        official,
        record,
        evidence,
        problems,
    }
}

/// A lookup with no recording says nothing about its source; only replay reaches this.
fn failure(error: Option<RequestError>, evidence: Vec<SourceOutcome>) -> Found {
    if error.and_then(status::reason) == Some(Reason::ReplayMiss)
        || evidence
            .iter()
            .any(|s| s.reason == Some(Reason::ReplayMiss))
    {
        return Found::Nothing;
    }
    Found::Failed(evidence)
}

fn lowered(name: &Text) -> String {
    name.as_str().to_lowercase()
}

/// The names one provider will search, and how many the record stated beyond them.
struct Capped {
    names: Vec<Text>,
    left_out: usize,
    too_long: usize,
    /// Official names past [`OFFICIAL_NAME_CEILING`], named in the notice.
    official_left_out: Vec<Text>,
}

impl Capped {
    fn message(&self, source: biomcp_types::Source, record: &str) -> String {
        let total = self.names.len() + self.left_out + self.too_long;
        let mut message = format!("Searched the names stated by {source} record {record}.");
        if self.too_long > 0 {
            message.push_str(&format!(
                " Left out {} of {total} names too long for the provider to search.",
                self.too_long
            ));
        }
        if self.left_out > 0 {
            message.push_str(&format!(
                " Left out {} of {total} names, past the drug_search_name_limit setting.",
                self.left_out
            ));
        }
        if !self.official_left_out.is_empty() {
            let names: Vec<_> = self.official_left_out.iter().map(Text::as_str).collect();
            message.push_str(&format!(
                " Left out official names past the ceiling of {OFFICIAL_NAME_CEILING}: {}.",
                names.join(", ")
            ));
        }
        message
    }
}

/// One spelling per name, the typed one kept when the provider can take it. Official names
/// pass the limit up to [`OFFICIAL_NAME_CEILING`]; the record's preferred names rank next and then the shortest, so the same
/// drug keeps the same names whichever of them was typed.
fn capped(
    names: Vec<Text>,
    (preferred, official): (&[String], &[String]),
    input: &Text,
    limit: usize,
    fits: fn(&str) -> bool,
) -> Capped {
    let mut seen = std::collections::BTreeSet::new();
    let stated: Vec<_> = std::iter::once(input.clone())
        .chain(names)
        .filter(|name| seen.insert(lowered(name)))
        .collect();
    let stated_count = stated.len();
    let mut names: Vec<_> = stated
        .into_iter()
        .filter(|name| fits(name.as_str()))
        .collect();
    let too_long = stated_count - names.len();
    let lowered_set = |names: &[String]| -> std::collections::BTreeSet<_> {
        names.iter().map(|name| name.to_lowercase()).collect()
    };
    let preferred = lowered_set(preferred);
    let official = lowered_set(official);
    names.sort_by_key(|name| {
        (
            !official.contains(&lowered(name)),
            !preferred.contains(&lowered(name)),
            name.as_str().chars().count(),
            lowered(name),
        )
    });
    let kept_official = names
        .iter()
        .filter(|name| official.contains(&lowered(name)))
        .count();
    let limit = limit.max(kept_official.min(OFFICIAL_NAME_CEILING));
    let left_out = names.len().saturating_sub(limit);
    let mut cut = names.split_off(limit.min(names.len()));
    // The typed name stays, in place of the last name that is not official, else the last one.
    if let Some(at) = cut.iter().position(|name| name == input) {
        let typed = cut.remove(at);
        let slot = names
            .iter()
            .rposition(|name| !official.contains(&lowered(name)))
            .or(names.len().checked_sub(1));
        if let Some(kept) = slot.and_then(|slot| names.get_mut(slot)) {
            cut.push(std::mem::replace(kept, typed));
        }
    }
    let official_left_out = cut
        .into_iter()
        .filter(|name| official.contains(&lowered(name)))
        .collect();
    Capped {
        names,
        left_out,
        too_long,
        official_left_out,
    }
}

fn record_evidence(
    out: &mut Outcome,
    evidence: Vec<SourceOutcome>,
    input: &Text,
    names: &[Text],
    message: String,
) {
    let mut noted = false;
    for mut source in evidence {
        if !noted {
            noted = true;
            source.notices.push(SourceNotice::DrugNameSearch {
                input: input.clone(),
                names: names.to_vec().into(),
                message: message.clone().into(),
            });
        }
        if let Some(existing) = out.sources.iter_mut().find(|s| s.source == source.source) {
            status::retain_source_state(existing, &source);
            existing.notices.extend(source.notices);
        } else {
            out.sources.push(source);
        }
    }
}

/// Keep disabled article providers and exact terms outside drug-name resolution.
pub(crate) fn article<'a>(
    cx: &Ctx<'a>,
    query: &biomcp_types::ArticleSearch,
    out: &mut Outcome,
    deadline: &mut Deadline<'a>,
) -> Resolved {
    if query.exact_term()
        || crate::article::next::search_sources(query)
            .iter()
            .all(|source| cx.fetch.disabled(*source))
    {
        return Resolved::default();
    }
    query
        .drug_name_input()
        .map_or_else(Resolved::default, |input| {
            bounded(deadline, |lookup| resolve(cx, input, out, lookup, |_| true))
        })
}

/// Whether the provider refused the request itself, as an oversized name group can make it do.
pub(crate) fn rejected(error: Option<&RequestError>) -> bool {
    matches!(
        error,
        Some(RequestError::Source(error)) if matches!(error.status, Some(400 | 413 | 414))
    )
}

/// The notice for a provider that searched only the typed term after refusing the names.
pub(crate) fn typed_only(input: &Text, source: biomcp_types::Source) -> SourceNotice {
    SourceNotice::DrugNameSearch {
        input: input.clone(),
        names: vec![input.clone()].into(),
        message: format!("{source} rejected the expanded request; searched the typed term.").into(),
    }
}

/// Stop claiming names no provider searched.
pub(crate) fn retract(out: &mut Outcome) {
    for notice in out.sources.iter_mut().flat_map(|s| &mut s.notices) {
        if let SourceNotice::DrugNameSearch {
            input,
            names,
            message,
        } = notice
        {
            *names = vec![input.clone()].into();
            *message =
                "The provider rejected the expanded request; searched the typed term.".into();
        }
    }
}

/// Providers taking one request per name get the typed name, then the others in `names` order.
pub(crate) fn one_request_each(input: Option<&Text>, names: &[Text]) -> Vec<Text> {
    let mut picked: Vec<_> = names.iter().collect();
    picked.sort_by_key(|name| Some(*name) != input);
    picked.truncate(NAME_REQUESTS);
    // Request order sets result order, so it must not depend on which name was typed.
    picked.sort_by_key(|name| name.as_str().to_lowercase());
    picked.into_iter().cloned().collect()
}

/// Source names are literal terms inside a native Boolean group.
pub(crate) fn alternatives(names: &[Text], input: &Text) -> Text {
    let mut names = names.to_vec();
    // A provider's reply is cached by its request, so the order must not depend on the typed name.
    names.sort_by_key(lowered);
    let terms: Vec<_> = names
        .iter()
        .map(|name| {
            format!(
                "\"{}\"",
                name.as_str().replace('\\', "\\\\").replace('"', "\\\"")
            )
        })
        .collect();
    Text::new(&format!("({})", terms.join(" OR "))).unwrap_or_else(|_| input.clone())
}
