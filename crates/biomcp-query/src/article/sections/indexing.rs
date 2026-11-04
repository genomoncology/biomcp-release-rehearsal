use biomcp_types::{ArticleGet, ArticleResolution, ArticleSectionWord, ReplyData, Section, Source};
pub(in crate::article) fn join(
    query: &ArticleGet,
    records: &[super::super::get::Record],
    out: &mut crate::Outcome,
) {
    if !super::selected(query, ArticleSectionWord::Indexing) {
        return;
    }
    let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    else {
        return;
    };
    for item in &mut cards.items {
        let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
            continue;
        };
        // Europe PMC owns its core cards even when they carry a related PMID.
        if card.europepmc_id.is_some() {
            continue;
        }
        let Some(section) = &mut item.sections else {
            continue;
        };
        if let Some((record, _)) = records.iter().find(|(r, _)| r.article.pmid == card.pmid) {
            let data = record.indexing.clone();
            section.indexing = Some(
                if data.mesh.is_empty()
                    && data.keywords.is_empty()
                    && data.affiliations.total == 0
                    && data.orcids.is_empty()
                {
                    Section::Empty {
                        source: Source::PubMed,
                        record: None,
                    }
                } else {
                    Section::Data {
                        source: Source::PubMed,
                        data,
                    }
                },
            );
        }
    }
}
