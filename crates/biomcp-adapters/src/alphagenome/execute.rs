//! Submit validated scoring requests through the bounded transport.
use super::{SCORE_VARIANT, ScoreReply, ScoreRequest};
/// Score one validated request using the transport leg's admission and recording policy.
pub fn score(
    leg: &biomcp_fetch::Leg<'_>,
    request: &ScoreRequest,
) -> Result<ScoreReply, biomcp_fetch::RequestError> {
    leg.grpc_checked(&SCORE_VARIANT, request.bytes(), |limits| {
        request.decoder(limits)
    })
}
