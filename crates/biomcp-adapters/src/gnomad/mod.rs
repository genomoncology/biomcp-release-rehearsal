//! gnomAD population constraint for checked human Ensembl identities.
use biomcp_fetch::{Input, Read};
mod map;
mod raw;
use crate::{Mapped, quotas};
use biodata::EnsemblGeneId;
use biomcp_fetch::{Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::{Constraint, Source};
use std::time::Duration;
/// Source policy for the gnomAD browser GraphQL endpoint.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC0 1.0",
        url: "https://gnomad.broadinstitute.org/policies",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Gnomad,
    base: "https://gnomad.broadinstitute.org",
    key: KeyUse::None,
    quota: &quotas::GNOMAD,
    deadline: Duration::from_secs(8),
};
const GENES: Endpoint = SPEC
    .post_json("/api")
    .idempotent()
    .fresh(Freshness::Days(7));
const FIELDS: &str = "gene_id canonical_transcript_id mane_select_transcript { ensembl_id ensembl_version } \
    gnomad_constraint { pli oe_lof oe_lof_upper mis_z syn_z flags }";

/// One requested identity and its independent checked section data.
#[derive(Debug)]
pub struct GeneConstraint {
    /// Submitted identity, retained independently of provider text.
    pub id: EnsemblGeneId,
    /// Source metrics, or a checked absence.
    pub constraint: Mapped<Option<Constraint>>,
}
/// Fetch sorted distinct identities in one bounded, read-only aliased query.
pub fn constraints(
    leg: &Leg<'_>,
    ids: &[EnsemblGeneId],
) -> Result<Vec<GeneConstraint>, RequestError> {
    let mut ids: Vec<_> = ids
        .iter()
        .map(|id| id.base().parse::<EnsemblGeneId>())
        .collect::<Result<_, _>>()
        .map_err(|_| biomcp_fetch::bad_reply(Source::Gnomad, None))?;
    ids.sort_unstable();
    ids.dedup();
    let body = crate::graphql::Query::aliases(ids.iter().map(ToString::to_string), |index| {
        format!("gene(gene_id: $id{index}, reference_genome: GRCh38) {{ {FIELDS} }}")
    });
    macro_rules! fetch {
        ($($n:literal),+) => {
            match ids.len() {
                $(
                    $n => leg.call(
                        &GENES,
                        Input::json(&[], &body),
                        Read::<raw::Genes<$n>>::json().cache_if(&|raw| raw.eligible(&ids)),
                    ).map(|received| received.value.data),
                )+
                _ => return Err(biomcp_fetch::bad_reply(Source::Gnomad, None)),
            }
        };
    }
    let raw = fetch!(
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
    )?;
    map::constraints(raw, &ids).map_err(|_| biomcp_fetch::bad_reply(Source::Gnomad, None))
}

/// Exact source-qualified population observations.
pub mod variant;

/// Complete source policy and command declaration.
pub(crate) mod decl;
