//! Validate shared human request identity before serializing either RPC.
use super::proto;
use biomcp_fetch::InputError;
pub(super) fn checked(
    build: biodata::Build,
    chromosome: &str,
    position: i64,
    reference: &str,
    alternate: &str,
) -> Result<proto::Variant, InputError> {
    if build != biodata::Build::GRCh38
        || position <= 0
        || biodata::Contig::new(chromosome).is_err()
        || chromosome
            .len()
            .saturating_add(reference.len())
            .saturating_add(alternate.len())
            > 4096
        || reference.is_empty()
        || alternate.is_empty()
        || !reference
            .bytes()
            .chain(alternate.bytes())
            .all(|b| b"ACGTN".contains(&b))
    {
        return Err(InputError::Path);
    }
    Ok(proto::Variant {
        chromosome: chromosome.into(),
        position,
        reference_bases: reference.into(),
        alternate_bases: alternate.into(),
    })
}
