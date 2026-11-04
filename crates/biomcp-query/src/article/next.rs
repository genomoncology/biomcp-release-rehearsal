use crate::{Action, Outcome};

use biomcp_types::{ArticleGet, ArticleResolution, ReplyData, Request, Status, StepKind};
use biomcp_types::{ArticleInput, Source};
pub(super) fn get(query: &ArticleGet, out: &mut Outcome) {
    if matches!(out.status(), Status::Failed | Status::Partial) {
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &get_sources(query),
            Request::GetArticle(query.clone()),
            true,
        ));
    }
    let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &out.body
    else {
        return;
    };
    for item in &cards.items {
        let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
            continue;
        };
        pdf(item, &mut out.next, &mut out.problems);
        let input = card
            .pmid
            .map(ArticleInput::Pmid)
            .or_else(|| card.europepmc_id.clone().map(ArticleInput::EuropePmc));
        let Some(input) = input else {
            continue;
        };
        if !item.more.is_empty() {
            let words = item
                .more
                .iter()
                .filter_map(|s| {
                    crate::next::checked(s.to_string().parse(), Source::PubMed, &mut out.problems)
                })
                .collect();
            if let Some(query) = crate::next::checked(
                ArticleGet::with_sections(vec![input].into(), words),
                biomcp_types::Source::PubMed,
                &mut out.problems,
            ) {
                push(
                    &mut out.next,
                    StepKind::Section,
                    &get_sources(&query),
                    Request::GetArticle(query),
                );
            }
        }
        let Some(id) = card.pmid else {
            continue;
        };
        push(
            &mut out.next,
            StepKind::Disambiguate,
            &[Source::Icite],
            Request::ArticleCitations(biomcp_types::ArticleEdges::new(id)),
        );

        push(
            &mut out.next,
            StepKind::Disambiguate,
            &[Source::SemanticScholar],
            Request::ArticleCitations(
                biomcp_types::ArticleEdges::new(id)
                    .sourced(biomcp_types::EdgeSource::SemanticScholar),
            ),
        );
        push(
            &mut out.next,
            StepKind::Disambiguate,
            &[Source::SemanticScholar],
            Request::ArticleRecommend(biomcp_types::ArticleRecommend::new(ArticleInput::Pmid(id))),
        );
    }
}
fn pdf(
    item: &biomcp_types::ArticleCardItem,
    next: &mut Vec<crate::next::Step>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
        return;
    };
    if item.sections.as_ref().is_some_and(|s| {
        matches!(
            s.fulltext,
            Some(
                biomcp_types::Section::NotApplicable
                    | biomcp_types::Section::Unavailable {
                        reason: biomcp_types::Reason::NotOpen,
                        ..
                    }
            )
        )
    }) && let Some(input) = card
        .pmid
        .map(ArticleInput::Pmid)
        .or_else(|| card.doi.clone().map(ArticleInput::Doi))
        && let Some(query) = crate::next::checked(
            ArticleGet::with_sections(
                vec![input].into(),
                vec![biomcp_types::ArticleSectionWord::Pdf].into(),
            ),
            biomcp_types::Source::PubMed,
            problems,
        )
    {
        push(
            next,
            StepKind::Section,
            &get_sources(&query),
            Request::GetArticle(query),
        );
    }
}

fn push(next: &mut Vec<crate::next::Step>, kind: StepKind, sources: &[Source], request: Request) {
    let step = crate::next::Step::run(kind, sources, request, false);
    if next.len() < 5 && !next.contains(&step) {
        next.push(step);
    }
}

/// A short complete pool offers caller-controlled alternatives without running them.
pub(super) fn search(
    query: &biomcp_types::ArticleSearch,
    total: u64,
    partial: bool,
    out: &mut Outcome,
) {
    if out.sources.iter().any(|source| {
        source.notices.iter().any(|notice| match notice {
            biomcp_types::SourceNotice::NotFoundWords { .. } => true,
            biomcp_types::SourceNotice::PubmedWarning { name, phrases } => {
                matches!(name.as_str(), "quotedphrasesnotfound" | "phrasesignored")
                    && !phrases.is_empty()
            }
            _ => false,
        })
    }) && let Some(strict) = query.strict_pubmed()
    {
        out.next.insert(
            0,
            crate::next::Step::new(
                StepKind::Disambiguate,
                &[Source::PubMed],
                Action::PubmedSearch(strict),
            ),
        );
    }
    if !partial && total < u64::from(query.limit()) {
        out.next.extend(query.term_drops().into_iter().map(|query| {
            crate::next::Step::run(
                StepKind::Disambiguate,
                &search_sources(&query),
                Request::SearchArticle(query),
                false,
            )
        }));
        if let Some(broader) = query.without_variant() {
            push(
                &mut out.next,
                StepKind::Disambiguate,
                &search_sources(&broader),
                Request::SearchArticle(broader),
            );
        }
    }
    variant_step(query, out);
}

fn variant_step(search: &biomcp_types::ArticleSearch, out: &mut Outcome) {
    if search.sources() == [biomcp_types::ArticleSource::Pubmed]
        && search.gene().is_some()
        && search.variant().is_some()
    {
        out.next.push(crate::next::Step::run(
            StepKind::Disambiguate,
            &[Source::PubMed, Source::LitVar],
            Request::SearchArticle(search.with_litvar()),
            false,
        ));
    }
}

/// Existing source-given author identities survive ordinary pool paging.
pub(super) fn authors(out: &mut Outcome) {
    let items = match &out.body {
        ReplyData::ArticlePage {
            body: Some(page), ..
        } => page.items.as_ref(),
        ReplyData::ArticleEdgePage {
            body: Some(page), ..
        } => page.items.as_ref(),
        ReplyData::ArticleRecommendations {
            body: Some(page), ..
        } => page.items.as_ref(),
        _ => return,
    };
    for id in items
        .iter()
        .filter_map(|row| row.header.as_ref())
        .filter_map(|header| header.first_author_id.as_ref())
    {
        let step = crate::next::Step::run(
            StepKind::Pivot,
            &[Source::SemanticScholar],
            Request::GetAuthor(biomcp_types::AuthorGet::one(
                biodata::AuthorId::SemanticScholar(id.clone()),
            )),
            false,
        );
        if !out.next.contains(&step) {
            out.next.push(step);
        }
    }
}

/// Unsearchable bodies still permit an explicit abstract read.
pub(super) fn text(query: &biomcp_types::ArticleText, out: &mut Outcome) {
    if matches!(&out.body, ReplyData::ArticleTextResult { body: Some(body), .. }
        if !matches!(body.content, biomcp_types::Section::Data { .. }))
        && let Some(get) = crate::next::checked(
            ArticleGet::new(vec![query.input().clone()].into()),
            biomcp_types::Source::PubMed,
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::run(
            StepKind::Disambiguate,
            &get_sources(&get),
            Request::GetArticle(get),
            false,
        ));
    }
}

/// Native abstract provider plus explicitly requested section providers.
pub(crate) fn get_sources(query: &ArticleGet) -> Vec<Source> {
    use biomcp_types::ArticleSectionWord as Section;
    let mut sources = query
        .input()
        .iter()
        .map(|input| match input {
            ArticleInput::EuropePmc(_) => Source::EuropePmc,
            _ => Source::PubMed,
        })
        .collect::<Vec<_>>();
    for section in query.sections() {
        match section {
            Section::Annotations => {
                sources.extend(biomcp_types::ArticleSection::Annotations.sources())
            }
            Section::Citations => sources.extend(biomcp_types::ArticleSection::Citations.sources()),
            Section::Tldr | Section::Pdf => {
                sources.extend(biomcp_types::ArticleSection::Tldr.sources())
            }
            Section::Supplements => {
                sources.extend(biomcp_types::ArticleSection::Supplements.sources())
            }
            Section::Fulltext => sources.extend(query.input().iter().map(|input| match input {
                ArticleInput::EuropePmc(_) => Source::EuropePmc,
                _ => Source::PmcOa,
            })),
            Section::All => sources.extend(
                [
                    biomcp_types::ArticleSection::Annotations,
                    biomcp_types::ArticleSection::Citations,
                    biomcp_types::ArticleSection::Tldr,
                    biomcp_types::ArticleSection::Supplements,
                ]
                .into_iter()
                .flat_map(biomcp_types::ArticleSection::sources),
            ),
            _ => {}
        }
    }
    sources
}
pub(crate) fn text_sources(input: &ArticleInput) -> Vec<Source> {
    match input {
        ArticleInput::EuropePmc(_) => vec![Source::EuropePmc],
        ArticleInput::Pmid(_) => vec![Source::PubMed, Source::PmcOa],
        _ => vec![Source::PmcOa],
    }
}
pub(crate) fn evidence_sources(query: &biomcp_types::ArticleEvidence) -> Vec<Source> {
    let mut sources = vec![Source::SemanticScholar];
    if !matches!(query.cited(), ArticleInput::Doi(_))
        || query.fulltext() && !matches!(query.citing(), ArticleInput::Pmcid(_))
    {
        sources.push(Source::PubMed);
    }
    if query.fulltext() {
        sources.push(Source::PmcOa);
    }
    sources
}
pub(crate) fn edge_source(query: &biomcp_types::ArticleEdges) -> Source {
    match query.source() {
        biomcp_types::EdgeSource::Icite => Source::Icite,
        biomcp_types::EdgeSource::SemanticScholar => Source::SemanticScholar,
    }
}
pub(crate) fn search_sources(query: &biomcp_types::ArticleSearch) -> Vec<Source> {
    query
        .sources()
        .iter()
        .map(|source| match source {
            biomcp_types::ArticleSource::Pubmed => Source::PubMed,
            biomcp_types::ArticleSource::Europepmc => Source::EuropePmc,
            biomcp_types::ArticleSource::Litvar => Source::LitVar,
            biomcp_types::ArticleSource::SemanticScholar => Source::SemanticScholar,
            biomcp_types::ArticleSource::Litsense => Source::LitSense,
            biomcp_types::ArticleSource::Pubtator => Source::PubTator,
        })
        .collect()
}
