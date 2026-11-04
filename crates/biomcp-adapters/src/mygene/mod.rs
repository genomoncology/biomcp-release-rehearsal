//! MyGene.info annotations queried only by checked Entrez gene identifiers.
use biomcp_fetch::{Input, Read};
mod map;
mod raw;
use crate::{Mapped, quotas};
use biodata::{GenomicInterval, HgncId, NcbiGeneId, Summary};

use biomcp_fetch::{Arg, Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::{Problem, Source};
use std::time::Duration;

/// Source declaration used to open a MyGene.info transport leg.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "MyGene.info Terms of Use; upstream data terms",
        url: "https://docs.mygene.info/en/latest/terms.html",
        usage: biomcp_types::Use::Open,
    },
    source: Source::MyGene,
    base: "https://mygene.info/v3",
    key: KeyUse::None,
    quota: &quotas::BIOTHINGS,
    deadline: Duration::from_secs(8),
};
const GENES: Endpoint = SPEC
    .post_form("/gene")
    .idempotent()
    .fresh(Freshness::Days(7));
const FIELDS: &str = "summary,genomic_pos,genomic_pos_hg19,HGNC,entrezgene";
// Local chunk size below the documented provider maximum.
const BATCH: usize = 200;

/// Source annotations with identities retained for the authoritative HGNC join.
#[derive(Debug)]
pub struct GeneExtras {
    /// Submitted query identifier; never inferred from array position.
    pub query_id: NcbiGeneId,
    /// Returned checked identifier, required to match the submitted query.
    pub entrez_id: NcbiGeneId,
    /// Returned HGNC identity, checked against the HGNC record by the caller.
    pub hgnc_id: Option<HgncId>,
    /// Whole RefSeq summary with only a recognized attribution marker removed.
    pub summary: Option<Summary>,
    /// Every accepted source interval, retaining its stated assembly.
    pub spans: Box<[GenomicInterval]>,
}
/// One submitted chunk and its typed outcome, independent of other chunks.
#[derive(Debug)]
pub struct GeneBatch {
    /// Sorted distinct identifiers submitted in this chunk.
    pub ids: Box<[NcbiGeneId]>,
    /// Accepted per-gene annotations or the original transport failure.
    pub result: Result<Vec<Mapped<GeneExtras>>, RequestError>,
    /// Rejected hits that have no safe per-gene value to carry their problems.
    pub problems: Vec<Problem>,
    /// Hits refused before an accepted per-gene annotation could be constructed.
    pub refusals: crate::Refusal,
}

/// Fetch annotations in sorted distinct chunks, preserving each chunk's failures.
pub fn genes(leg: &Leg<'_>, ids: &[NcbiGeneId]) -> Vec<GeneBatch> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let chunks: Vec<_> = ids.chunks(BATCH).collect();
    let results = leg.batch(&chunks, |leg, chunk| {
        let ids: Vec<Box<str>> = chunk.iter().map(|id| id.to_string().into()).collect();
        leg.call(
            &GENES,
            Input::Params(&[
                ("ids", Arg::Ids(&ids)),
                ("fields", Arg::Text(FIELDS)),
                ("species", Arg::Text("human")),
            ]),
            Read::<raw::Genes>::json(),
        )
        .map(|received| received.value)
    });
    chunks
        .into_iter()
        .zip(results)
        .map(|(ids, result)| {
            let mut problems = Vec::new();
            let mut refusals = crate::Refusal::None;
            let result = result.map(|reply| map::extras(reply, ids, &mut problems, &mut refusals));
            GeneBatch {
                ids: ids.into(),
                result,
                problems,
                refusals,
            }
        })
        .collect()
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
