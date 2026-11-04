//! Resolve source notation through CAR without interpreting or repairing its allele.
use crate::{Ctx, status};
use biodata::{Build, Chromosome, ExpressionSyntax};
use biomcp_adapters::allele_registry;
use biomcp_adapters::alphagenome::ScoreRequest;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{Problem, Source, SourceOutcome, VariantHeader};

#[derive(Debug)]
pub(super) struct Joined {
    pub(super) request: Option<ScoreRequest>,
    pub(super) sources: Vec<SourceOutcome>,
    pub(super) problems: Vec<Problem>,
    pub(super) local: Option<RequestError>,
    pub(super) unsupported: bool,
}
pub(super) fn resolve(cx: &Ctx<'_>, row: &VariantHeader, deadline: &mut Deadline<'_>) -> Joined {
    let mut joined = Joined {
        request: super::alphagenome::request(row),
        sources: Vec::new(),
        problems: Vec::new(),
        local: None,
        unsupported: mitochondrial(row),
    };
    if joined.request.is_some() || joined.unsupported {
        return joined;
    }
    let Some(notation) = notation(row) else {
        return joined;
    };
    let leg = cx
        .fetch
        .leg(&allele_registry::SPEC, *deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let reply = allele_registry::hgvs(&leg, &notation);
    let error = reply.as_ref().err().copied();
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    joined
        .sources
        .push(status::source(Source::AlleleRegistry, leg.finish(), error));
    joined.local = error.and_then(status::local);
    match reply {
        Ok(allele_registry::Resolution::Found(row)) => {
            joined.unsupported = mitochondrial(&row.value);
            joined.request = super::alphagenome::request(&row.value);
            joined.problems = row.problems;
        }
        Ok(allele_registry::Resolution::Refused(problems)) => joined.problems = problems,
        _ => (),
    }
    joined
}
fn mitochondrial(row: &VariantHeader) -> bool {
    row.identity
        .representations
        .iter()
        .filter_map(|representation| representation.allele.as_ref())
        .any(|allele| {
            Chromosome::from_accession(allele.location().sequence, Build::GRCh38)
                .is_ok_and(|chromosome| chromosome.number() == 25)
        })
}
fn notation(row: &VariantHeader) -> Option<String> {
    if row.source != Source::MyVariant {
        return None;
    }
    let [representation] = &*row.identity.representations else {
        return None;
    };
    if representation.build != Some(Build::GRCh38)
        || representation.expression.syntax() != ExpressionSyntax::HgvsG
    {
        return None;
    }
    let (reference, body) = representation.expression.value().as_str().split_once(':')?;
    let chromosome: Chromosome = reference.strip_prefix("chr")?.parse().ok()?;
    // Preserve the source's complete g. expression. The existing assembly map only
    // qualifies its chromosome name for CAR's reference-qualified input contract.
    Some(format!("{}:{body}", chromosome.accession(Build::GRCh38)))
}
