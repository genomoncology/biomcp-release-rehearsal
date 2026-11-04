use crate::{Ctx, Outcome, gene, status};

use biomcp_adapters::hgnc;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{ArticleSearch, Request, Source, StepKind};
use biomcp_types::{GeneInput, GeneResolution};
pub(super) struct Resolved {
    pub names: Vec<String>,
    pub symbol: Option<String>,
    pub entrez: Option<biodata::NcbiGeneId>,
}
pub(super) fn resolve<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSearch,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Option<Resolved>, RequestError> {
    let Some(input) = query.gene() else {
        return Ok(Some(Resolved {
            names: Vec::new(),
            symbol: None,
            entrez: None,
        }));
    };
    let leg = cx.fetch.leg(&hgnc::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let resolved = gene::resolve(&leg, input);
    *deadline = super::consume(*deadline, started, leg.remaining());
    out.sources.push(status::source(
        Source::Hgnc,
        leg.finish(),
        resolved.as_ref().err().copied(),
    ));
    let mapped = resolved?;
    status::append(&mut out.problems, mapped.problems);
    match mapped.value {
        GeneResolution::Resolved { gene, .. } => {
            let mut names = vec![gene.symbol.to_string()];
            let typed = match input {
                GeneInput::Text(text) => Some(text.as_str()),
                _ => None,
            };
            for name in typed.into_iter().map(str::to_owned) {
                if !names.iter().any(|n| n.eq_ignore_ascii_case(&name)) {
                    names.push(name);
                }
            }
            let symbol = gene.symbol.to_string();
            Ok(Some(Resolved {
                names,
                symbol: Some(symbol),
                entrez: gene.entrez_id,
            }))
        }
        GeneResolution::Ambiguous { candidates } => {
            for candidate in candidates.iter().take(5) {
                let checked = query.with_gene(GeneInput::HgncId(candidate.hgnc_id));
                out.next.push(crate::next::Step::run(
                    StepKind::Disambiguate,
                    &super::next::search_sources(&checked),
                    Request::SearchArticle(checked),
                    cx.fresh,
                ));
            }
            Ok(None)
        }
        _ => {
            crate::status::item(out, false, false, false);
            Ok(None)
        }
    }
}
