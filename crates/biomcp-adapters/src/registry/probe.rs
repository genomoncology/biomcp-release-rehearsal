//! Health requests come from the owning adapter.
/// Run the explicitly declared bounded source probe.
pub fn probe(
    spec: &'static biomcp_fetch::SourceSpec,
    leg: &biomcp_fetch::Leg<'_>,
) -> Result<(), biomcp_fetch::RequestError> {
    crate::decl::decl(spec.source).probe.run(spec, leg)
}
