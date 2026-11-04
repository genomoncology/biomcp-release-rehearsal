//! Article section preparation and dispatch, following the gene section pattern.
mod annotations;
mod citations;
pub(crate) mod fulltext;
pub(super) mod indexing;
pub(in crate::article) mod pdf;
mod tldr;
use crate::{Ctx, status};

use biomcp_fetch::Deadline;
use biomcp_types::{
    ArticleCardItem, ArticleGet, ArticleResolution, ArticleSection, ArticleSectionWord,
    ArticleSections, Section,
};
pub(super) fn prepare(query: &ArticleGet, items: &mut [ArticleCardItem]) -> bool {
    let requested = !query.sections().is_empty();
    for item in items {
        let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
            continue;
        };
        if requested {
            item.sections = Some(ArticleSections {
                pdf: (selected(query, ArticleSectionWord::Pdf)
                    && card.pmid.is_none()
                    && card.doi.is_none())
                .then_some(Section::NotApplicable),
                indexing: selected(query, ArticleSectionWord::Indexing)
                    .then_some(Section::NotApplicable),
                supplements: (selected(query, ArticleSectionWord::Supplements)
                    && card.pmcid.is_none())
                .then_some(Section::NotApplicable),
                tldr: (selected(query, ArticleSectionWord::Tldr)
                    && card.pmid.is_none()
                    && card.doi.is_none())
                .then_some(Section::NotApplicable),
                annotations: (selected(query, ArticleSectionWord::Annotations)
                    && card.pmid.is_none())
                .then_some(Section::NotApplicable),
                citations: (selected(query, ArticleSectionWord::Citations) && card.pmid.is_none())
                    .then_some(Section::NotApplicable),
                fulltext: (selected(query, ArticleSectionWord::Fulltext)
                    && card.pmcid.is_none()
                    && card.europepmc_id.is_none())
                .then_some(Section::NotApplicable),
            });
        }
        item.more = [
            (
                ArticleSectionWord::Indexing,
                ArticleSection::Indexing,
                card.pmid.is_some() && card.europepmc_id.is_none(),
            ),
            (
                ArticleSectionWord::Annotations,
                ArticleSection::Annotations,
                card.pmid.is_some(),
            ),
            (
                ArticleSectionWord::Fulltext,
                ArticleSection::Fulltext,
                card.pmcid.is_some() || card.europepmc_id.is_some(),
            ),
            (
                ArticleSectionWord::Supplements,
                ArticleSection::Supplements,
                card.pmcid.is_some(),
            ),
            (
                ArticleSectionWord::Citations,
                ArticleSection::Citations,
                card.pmid.is_some(),
            ),
            (
                ArticleSectionWord::Tldr,
                ArticleSection::Tldr,
                card.pmid.is_some() || card.doi.is_some(),
            ),
            (
                ArticleSectionWord::Pdf,
                ArticleSection::Pdf,
                card.pmid.is_some() || card.doi.is_some(),
            ),
        ]
        .into_iter()
        .filter(|(word, _, available)| *available && !selected(query, *word))
        .map(|(_, section, _)| section)
        .collect();
    }
    requested
}
pub(super) fn run<'a>(
    cx: &Ctx<'a>,
    deadline: &mut Deadline<'a>,
    query: &ArticleGet,
    out: &mut crate::Outcome,
) {
    let fulltext = selected(query, ArticleSectionWord::Fulltext);
    let supplements = selected(query, ArticleSectionWord::Supplements);
    if fulltext || supplements {
        fulltext::run(cx, deadline, out, fulltext, supplements);
        if fulltext {
            super::preprint::fulltext(cx, deadline, out);
        }
    }
    if selected(query, ArticleSectionWord::Tldr) {
        tldr::run(cx, deadline, out);
    }
    if selected(query, ArticleSectionWord::Pdf) {
        pdf::run(cx, deadline, out);
    }
    let biomcp_types::ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    else {
        return;
    };
    let ids = annotations::ids(&cards.items);
    if ids.is_empty() {
        return;
    }
    if selected(query, ArticleSectionWord::Annotations) {
        let result = annotations::run(cx, deadline, &ids);
        annotations::join(&mut cards.items, &result);
        out.local = out.local.or(result.local);
        let state = status::status(cards.items.iter().map(|i| i.status));
        status::get(out, [state].into_iter());
        status::append(&mut out.problems, result.problems);
        out.sources.push(result.source);
    }
    if selected(query, ArticleSectionWord::Citations) {
        citations::run(cx, deadline, &ids, out);
    }
}
fn selected(query: &ArticleGet, word: ArticleSectionWord) -> bool {
    query.sections().contains(&ArticleSectionWord::All) || query.sections().contains(&word)
}
