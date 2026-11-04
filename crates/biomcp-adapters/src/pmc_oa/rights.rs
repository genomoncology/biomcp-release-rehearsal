//! Per-record rights admission is repeated when reading cached metadata.
use biodata::License;
use biomcp_fetch::{Leg, RequestError, SourceError};
use biomcp_types::{Source, Use};

/// Commercial reuse is allowed by these explicit record licenses.
pub fn commercial(license: License) -> bool {
    matches!(
        license,
        License::CcBy | License::CcBySa | License::CcByNd | License::Cc0 | License::PublicDomain
    )
}

/// Hosted restricted and commercial calls never expose unlicensed full text.
pub fn permitted(leg: &Leg<'_>, source: Source, license: License) -> Result<(), RequestError> {
    if leg.disabled_use(Use::PerRecord) && !commercial(license) {
        return Err(SourceError::disabled(source).into());
    }
    Ok(())
}
