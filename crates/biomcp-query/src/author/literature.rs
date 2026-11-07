//! Citation author discovery does not infer identities from matching names.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::pubmed;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    AuthorLiterature, AuthorPage, AuthorPublication, AuthorSearch, AuthorSearchSource, PageInfo,
    Paged, ReplyBody, Request, Source, StepKind,
};

/// Names shown per paper, matching the article card; one consortium paper lists thousands.
const AUTHOR_NAMES: usize = 10;

pub(super) fn search(cx: &Ctx<'_>, q: &AuthorSearch) -> Outcome {
    if q.pool().is_some() {
        let mut out = status::empty(biomcp_types::ReplyData::PoolError { mismatch: true });
        out.next.push(crate::next::Step::new(
            StepKind::Restart,
            &[Source::PubMed],
            super::run(Request::SearchAuthor(q.restart())),
        ));
        return out;
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = status::empty(AuthorPage::into_reply(q.clone(), None));
    if let Err(error) = execute(cx, q, deadline, &mut out) {
        out.local = status::local(error);
    }
    command.finish(deadline.remaining());
    out
}

fn execute(
    cx: &Ctx<'_>,
    q: &AuthorSearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&pubmed::SPEC, deadline).fresh(cx.fresh);
    let mut notices = Vec::new();
    let mut count = None;
    let result = (|| {
        let found = pubmed::search_ids(
            &leg,
            q.query(),
            biomcp_types::ArticleSort::Rank,
            cx.ncbi_email.as_ref(),
        )?;
        out.problems.extend(found.problems);
        notices = found.notices;
        count = Some(found.count);
        let ids: Vec<_> = found
            .ids
            .into_iter()
            .skip(q.offset() as usize)
            .take(q.limit() as usize)
            .collect();
        let mapped = if ids.is_empty() {
            None
        } else {
            Some(pubmed::author_summaries(
                &leg,
                &ids,
                cx.ncbi_email.as_ref(),
            )?)
        };
        let papers = if let Some(mapped) = mapped {
            out.include_refusals(mapped.refusals);
            out.problems.extend(mapped.problems);
            mapped.value
        } else {
            Vec::new()
        };
        present(q, out, papers, found.count, ids.len() as u32, &mut notices)
    })();
    let error = result.as_ref().err().copied();
    let mut source = status::source(Source::PubMed, leg.finish(), error);
    source.term = match q.query().parse() {
        Ok(term) => Some(term),
        Err(_) => {
            out.problems.push(biomcp_types::Problem {
                source: Source::PubMed,
                record: None,
                field: "term".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
            None
        }
    };
    source.count = count;
    source.notices = notices;
    out.sources.push(source);
    // Offset stays: no pool ties this page, and a step naming a disabled source is dropped.
    super::retry(out, Request::SearchAuthor(q.clone()));
    // The source can explain PubMed's query interpretation without claiming identity resolution.
    if q.source() == AuthorSearchSource::Auto && cx.fetch.disabled(Source::SemanticScholar) {
        out.sources.push(biomcp_adapters::registry::outcome(
            Source::SemanticScholar,
            biomcp_types::SourceStatus::Skipped(Some(biomcp_types::Reason::Disabled)),
        ));
    }
    if result.is_ok() {
        status::item(out, true, false, false);
    }
    result
}

fn present(
    q: &AuthorSearch,
    out: &mut Outcome,
    mut papers: Vec<AuthorPublication>,
    count: u64,
    consumed: u32,
    notices: &mut Vec<biomcp_types::SourceNotice>,
) -> Result<(), RequestError> {
    let returned = papers.len() as u32;
    for paper in &mut papers {
        bound(q.query(), paper);
        if let (Some(omitted), Some(pmid)) = (paper.authors_omitted, paper.paper.pmid.as_ref()) {
            notices.push(biomcp_types::SourceNotice::RequestGuidance {
                message: format!(
                    "Paper {pmid}: authors omitted: {omitted}; read every author at \
                     https://pubmed.ncbi.nlm.nih.gov/{pmid}/."
                )
                .into(),
                retry_after_seconds: None,
                data_date: None,
            });
        }
    }
    if q.offset().saturating_add(consumed) < count.min(9999) as u32 {
        let next = q.with_source(AuthorSearchSource::Pubmed);
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[Source::PubMed],
            super::run(Request::SearchAuthor(
                next.at_offset(q.offset() + q.limit()),
            )),
        ));
    }
    out.page = Some(PageInfo {
        offset: q.offset(),
        returned,
        total: count,
        pool: None,
        truncated: (count > 9999).then_some(true),
    });
    out.body = AuthorPage {
        items: Box::default(),
        matching_records: None,
        literature: Some(AuthorLiterature {
            identity_resolution: concat!(
                "Unresolved: these are author names on individual papers, not verified researcher identities. ",
                "Equal names may refer to different people; names are not merged. ",
                "Affiliations, author-specific publication totals, citation metrics and topic expertise are not established. ",
                "PubMed matches papers, not people; only the first 9999 matching papers can be paged."
            ).parse().map_err(|_| biomcp_fetch::CacheError::Encoding)?,
            papers: papers.into(),
        }),
    }.with_query(q.clone());
    Ok(())
}

/// Keep source order, showing names like the query first (best effort) and then the leading names.
fn bound(query: &str, paper: &mut AuthorPublication) {
    let total = paper.authors.len();
    if total <= AUTHOR_NAMES {
        return;
    }
    let query = words(query);
    let mut keep = vec![false; total];
    let matched = paper
        .authors
        .iter()
        .enumerate()
        .filter(|(_, name)| same_words(&query, name.as_str()))
        .take(AUTHOR_NAMES)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for index in &matched {
        if let Some(slot) = keep.get_mut(*index) {
            *slot = true;
        }
    }
    keep.iter_mut()
        .filter(|slot| !**slot)
        .take(AUTHOR_NAMES - matched.len())
        .for_each(|slot| *slot = true);
    let mut keep = keep.into_iter();
    let shown: Vec<_> = std::mem::take(&mut paper.authors)
        .into_vec()
        .into_iter()
        .filter(|_| keep.next().unwrap_or(false))
        .collect();
    paper.authors_omitted = Some((total - shown.len()) as u64);
    paper.authors = shown.into();
}

fn words(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| {
            word.chars()
                .flat_map(char::to_lowercase)
                .fold(String::new(), |mut out, ch| {
                    match fold(ch) {
                        Some(base) => out.push_str(base),
                        None => out.push(ch),
                    }
                    out
                })
        })
        .collect()
}

/// Base letters for lowercase Latin-1 and Latin Extended-A, from Unicode decompositions, so
/// `Müller` and `Muller` compare equal on both sides; names are lowercased first.
fn fold(ch: char) -> Option<&'static str> {
    Some(match ch {
        'ß' => "ss",
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => "a",
        'æ' => "ae",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => "e",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => "i",
        'ð' | 'ď' | 'đ' => "d",
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŉ' | 'ŋ' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => "o",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
        'ý' | 'ÿ' | 'ŷ' => "y",
        'þ' => "th",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
        'ĥ' | 'ħ' => "h",
        'ĵ' => "j",
        'ķ' | 'ĸ' => "k",
        'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
        'œ' => "oe",
        'ŕ' | 'ŗ' | 'ř' => "r",
        'ś' | 'ŝ' | 'ş' | 'š' => "s",
        'ţ' | 'ť' | 'ŧ' => "t",
        'ŵ' => "w",
        'ź' | 'ż' | 'ž' => "z",
        _ => return None,
    })
}

/// Best effort: PubMed does not say which name matched and citation names carry initials,
/// so every accent-folded query word must pair with its own name word, one a prefix of the
/// other. A miss only changes which names are shown; the notice still counts the omitted
/// names and links the full list. It claims no identity.
fn same_words(query: &[String], name: &str) -> bool {
    let mut name = words(name);
    !query.is_empty()
        && query.iter().all(|word| {
            name.iter()
                .position(|part| part.starts_with(word.as_str()) || word.starts_with(part.as_str()))
                .map(|index| name.swap_remove(index))
                .is_some()
        })
}
