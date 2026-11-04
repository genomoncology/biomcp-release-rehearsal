//! Require a caller key and a source-stated genomic allele before scoring.
use crate::{Ctx, status};
use biodata::{Build, Chromosome};
use biomcp_adapters::alphagenome;
use biomcp_adapters::alphagenome::ScoreRequest;
use biomcp_fetch::{Deadline, SecretName};
use biomcp_types::{
    Reason, Section, Source, SourceOutcome, SourceState, VariantAlphaGenome, VariantGet,
    VariantHeader,
};

pub(super) fn section(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &[VariantHeader],
    mut deadline: Deadline<'_>,
) -> super::sections::Result<VariantAlphaGenome> {
    if !cx.fetch.has_key(SecretName::AlphaGenomeApiKey) {
        return refused(Reason::NoKey, SourceState::Skipped);
    }
    if query.build().is_some_and(|build| build != Build::GRCh38) {
        return refused(Reason::Unsupported, SourceState::Failed);
    }
    let Some(candidate) = candidates.first() else {
        return refused(Reason::NoJoinId, SourceState::Failed);
    };
    let joined = super::alphagenome_join::resolve(cx, candidate, &mut deadline);
    let Some(request) = joined.request else {
        let reason = if joined.unsupported
            || (candidate
                .identity
                .representations
                .iter()
                .any(|r| r.build.is_some())
                && !candidate
                    .identity
                    .representations
                    .iter()
                    .any(|r| r.build == Some(Build::GRCh38)))
        {
            Reason::Unsupported
        } else {
            Reason::NoJoinId
        };
        let (section, mut sources, _, _, _) = refused(reason, SourceState::Failed);
        sources.extend(joined.sources);
        return (section, sources, joined.problems, joined.local, false);
    };
    if request.bytes().len() > biomcp_adapters::alphagenome::SCORE_LIMITS.request_bytes {
        let (section, mut sources, _, _, _) = refused(Reason::TooLarge, SourceState::Failed);
        sources.extend(joined.sources);
        return (section, sources, joined.problems, joined.local, false);
    }
    let leg = cx.fetch.leg(&alphagenome::SPEC, deadline).fresh(cx.fresh);
    let reply = alphagenome::score(&leg, &request)
        .and_then(|reply| super::alphagenome_page::page(query, &request, reply));
    let error = reply.as_ref().err().copied();
    let dropped = reply
        .as_ref()
        .is_ok_and(|mapped| mapped.refusals == biomcp_adapters::Refusal::Refused);
    let source = status::source(Source::AlphaGenome, leg.finish(), error);
    let mut problems = joined.problems;
    let section = match reply {
        Ok(data) => {
            problems.extend(data.problems);
            Section::Data {
                source: Source::AlphaGenome,
                data: data.value,
            }
        }
        Err(error) => Section::unavailable(
            Source::AlphaGenome,
            status::reason(error).unwrap_or(Reason::Unavailable),
        ),
    };
    (
        section,
        joined.sources.into_iter().chain([source]).collect(),
        problems,
        joined.local.or(error.and_then(status::local)),
        dropped,
    )
}

pub(super) fn request(row: &VariantHeader) -> Option<ScoreRequest> {
    let mut requests = row
        .identity
        .representations
        .iter()
        .filter_map(|representation| {
            if representation.build != Some(Build::GRCh38) {
                return None;
            }
            let allele = representation.allele.as_ref()?;
            let chromosome =
                Chromosome::from_accession(allele.location().sequence, Build::GRCh38).ok()?;
            ScoreRequest::human_legacy(
                Build::GRCh38,
                chromosome,
                i64::from(allele.location().span.start()) + 1,
                &allele.reference().to_string(),
                &allele.alternate().to_string(),
            )
            .ok()
        });
    let first = requests.next()?;
    // Multiple source representations may repeat one identity, but never select between alleles.
    requests
        .all(|request| request.bytes() == first.bytes())
        .then_some(first)
}

fn refused(reason: Reason, state: SourceState) -> super::sections::Result<VariantAlphaGenome> {
    let section = if state == SourceState::Skipped {
        Section::Skipped {
            source: Source::AlphaGenome,
            reason,
        }
    } else {
        Section::unavailable(Source::AlphaGenome, reason)
    };
    (
        section,
        vec![biomcp_adapters::registry::outcome(
            Source::AlphaGenome,
            state,
            Some(reason),
        )],
        Vec::new(),
        None,
        false,
    )
}

pub(super) fn next(sources: &[SourceOutcome]) -> Vec<crate::next::Step> {
    if sources
        .iter()
        .any(|source| source.source == Source::AlphaGenome && source.reason == Some(Reason::NoKey))
    {
        vec![crate::next::Step::new(
            biomcp_types::StepKind::Key,
            &[biomcp_types::Source::AlphaGenome],
            crate::Action::KeyHelp(biomcp_fetch::SecretName::AlphaGenomeApiKey),
        )]
    } else {
        Vec::new()
    }
}
