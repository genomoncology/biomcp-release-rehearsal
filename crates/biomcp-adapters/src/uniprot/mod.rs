//! Reviewed human proteins fetched only by HGNC-listed accessions.
use biomcp_fetch::{Input, Read};
mod entry;
mod map;
mod raw;
mod search;
mod sequence;
use crate::{Mapped, quotas};
use biodata::UniProtAcc;
use biomcp_fetch::{Arg, Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::{Protein, Source};
pub use entry::{Entry, entry};
pub use search::{SearchPage, search};
pub use sequence::sequence;
use std::time::Duration;
/// Source policy for UniProt's accession endpoint.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC BY 4.0",
        url: "https://www.uniprot.org/help/license",
        usage: biomcp_types::Use::Attribution,
    },
    source: Source::UniProt,
    base: "https://rest.uniprot.org",
    key: KeyUse::None,
    quota: &quotas::UNIPROT,
    deadline: Duration::from_secs(8),
};
const PROTEINS: Endpoint = SPEC
    .get("/uniprotkb/accessions")
    .identity_encoding()
    .fresh(Freshness::Days(7));
const FIELDS: &str = "accession,protein_name,length,cc_function,cc_subcellular_location,reviewed,gene_primary,organism_id";
/// Fetch distinct accessions once, correlating checked replies by identity.
pub fn proteins(leg: &Leg<'_>, ids: &[UniProtAcc]) -> Result<Mapped<Vec<Protein>>, RequestError> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids.dedup();
    let text: Vec<Box<str>> = ids.iter().map(|id| id.to_string().into()).collect();
    let raw = leg
        .call(
            &PROTEINS,
            Input::Params(&[
                ("accessions", Arg::Ids(&text)),
                ("fields", Arg::Text(FIELDS)),
                ("format", Arg::Text("json")),
            ]),
            Read::<raw::Proteins>::json(),
        )?
        .value;
    Ok(map::proteins(raw, &ids))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
