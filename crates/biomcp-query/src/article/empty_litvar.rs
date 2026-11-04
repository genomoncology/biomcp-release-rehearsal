//! Recover an empty LitVar selection without inferring another variant identity.
use crate::Outcome;
use biomcp_types::{ArticleSearch, ArticleSource, Request, SourceNotice, StepKind};

pub(super) fn recover(query: &ArticleSearch, out: &mut Outcome) {
    for source in [ArticleSource::Europepmc, ArticleSource::Pubmed] {
        out.next.push(crate::next::Step::run(
            StepKind::Disambiguate,
            &super::next::search_sources(&query.with_source(source)),
            Request::SearchArticle(query.with_source(source)),
            false,
        ));
    }
    if let Some(input) = query
        .variant()
        .filter(|value| biomcp_adapters::litvar::is_rsid(value.as_str()))
        .and_then(|value| {
            crate::next::checked(
                value.as_str().parse(),
                biomcp_types::Source::LitVar,
                &mut out.problems,
            )
        })
        && let Some(get) = crate::next::checked(
            biomcp_types::VariantGet::new(input, biomcp_types::VariantResolver::Car, None),
            biomcp_types::Source::LitVar,
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::run(
            StepKind::Disambiguate,
            &[biomcp_types::Source::MyVariant],
            Request::GetVariant(get),
            false,
        ));
    }
    if let Some(broader) = query.without_variant() {
        out.next.push(crate::next::Step::run(
            StepKind::Disambiguate,
            &super::next::search_sources(&broader),
            Request::SearchArticle(broader),
            false,
        ));
    }
    if let Some(source) = out.sources.last_mut() {
        source.notices.push(SourceNotice::LitvarRsidHint {
            message:
                "If you have an rsID, look it up in LitVar with --variant RSID --source litvar."
                    .into(),
        });
    }
}
