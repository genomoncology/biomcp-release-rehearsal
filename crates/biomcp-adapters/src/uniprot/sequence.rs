//! An alternative sequence is admitted only under its exact native FASTA identity.
use super::SPEC;
use biodata::{AminoAcids, ProteinAudit, ProteinSequence, UniProtIsoform};
use biomcp_fetch::{Freshness, Leg, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::Reason;
/// Retrieve a direct alternative sequence; never construct it from displayed coordinates.
pub fn sequence(leg: &Leg<'_>, id: &UniProtIsoform) -> Result<ProteinSequence, RequestError> {
    let ep = SPEC
        .get("")
        .with_path(format!("/uniprotkb/{id}.fasta"))?
        .bytes(1024 * 1024)
        .identity_encoding()
        .fresh(Freshness::Days(7));
    let bytes = leg
        .call(
            &ep,
            Input::Params(&[]),
            Read::bytes()
                .validate_body(&|bytes| parse(bytes, id).map(|_| ()).ok_or(Reason::BadReply)),
        )
        .map(|received| received.value.0)?;
    parse(&bytes, id).ok_or(RequestError::Source(biomcp_fetch::SourceError {
        source: biomcp_types::Source::UniProt,
        reason: Reason::BadReply,
        status: None,
        tls_cause: None,
        retry_after: None,
    }))
}
fn parse(bytes: &[u8], id: &UniProtIsoform) -> Option<ProteinSequence> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut lines = text.lines();
    let header = lines.next()?;
    let mut fields = header.strip_prefix(">sp|")?.split('|');
    if fields.next()? != id.as_str() || !header.contains(" OX=9606 ") {
        return None;
    }
    let sequence = lines.collect::<String>();
    let sequence = AminoAcids::new(&sequence).ok()?;
    let length = u32::try_from(sequence.as_str().len()).ok()?;
    Some(ProteinSequence {
        accession: id.clone(),
        sequence,
        length,
        crc64: None,
        md5: None,
        audit: ProteinAudit {
            entry_version: None,
            sequence_version: None,
            first_public: None,
            last_annotation_update: None,
            last_sequence_update: None,
            release: None,
        },
    })
}
