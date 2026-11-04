//! NCBI EFetch sequence parameters: https://www.ncbi.nlm.nih.gov/books/NBK25499/.
//! seq_start and seq_stop are inclusive, one-based; strand=1 is the forward reference.
use super::FETCH;
const REFERENCE: biomcp_fetch::Endpoint = FETCH.bytes(1024);
use biomcp_fetch::{Arg, BadReply, Bytes, Input, Leg, Read, RequestError};

/// Read at most one reference base, only for a source-qualified literal SNV.
pub(crate) fn reference_agrees(
    leg: &Leg<'_>,
    chromosome: &str,
    build: biodata::Build,
    position: u64,
    reference: &str,
) -> Result<bool, RequestError> {
    let chromosome = biodata::Chromosome::parse(if chromosome == "M" { "MT" } else { chromosome })
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::ClinVar, None))?;
    let accession = chromosome.accession(build).to_string();
    let params = [
        ("db", Arg::Text("nuccore")),
        ("id", Arg::Text(&accession)),
        ("rettype", Arg::Text("fasta")),
        ("retmode", Arg::Text("text")),
        ("strand", Arg::Num(1)),
        ("seq_start", Arg::Num(position)),
        ("seq_stop", Arg::Num(position)),
        ("tool", Arg::Text("biomcp")),
    ];
    let reply = leg.call(
        &REFERENCE,
        Input::Params(&params),
        Read::<Bytes>::bytes().check(&|bytes| fasta(&bytes.0, &accession, position).map(|_| ())),
    )?;
    Ok(fasta(&reply.value.0, &accession, position)
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::ClinVar, None))?
        == reference)
}
fn fasta<'a>(bytes: &'a [u8], accession: &str, position: u64) -> Result<&'a str, BadReply> {
    let text = std::str::from_utf8(bytes).map_err(|_| BadReply)?;
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or(BadReply)?
        .split_whitespace()
        .next()
        .ok_or(BadReply)?;
    if header != format!(">{accession}:{position}-{position}") {
        return Err(BadReply);
    }
    let base = lines.next().ok_or(BadReply)?;
    if base.len() != 1
        || !base.bytes().all(|b| b"ACGT".contains(&b))
        || lines.any(|line| !line.is_empty())
    {
        return Err(BadReply);
    }
    Ok(base)
}
