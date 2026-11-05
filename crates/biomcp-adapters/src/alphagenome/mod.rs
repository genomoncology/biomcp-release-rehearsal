//! Encode AlphaGenome requests and admit complete bounded provider replies.
mod envelope;
mod envelope_budget;
mod preflight;
mod proto;
mod request;
mod request_variant;
mod score;
mod score_envelope;
mod score_metadata;
mod score_reply;
mod tensor;
mod wire;
use biomcp_fetch::grpc::{CodecError, Limits, StreamDecoder};
pub use envelope::Envelope;
pub use request::{DnaseRequest, SCHEMA};
pub use score::{SCORE_SCHEMA, ScoreRequest};
pub use score_envelope::{SCORE_LIMITS, ScoreEnvelope};
pub use score_reply::{ScoreGroup, ScoreReply};
pub use tensor::{DataType, Tensor, TensorStream};

mod execute;
pub use execute::score;

use biomcp_fetch::{Credential, KeyPlace, KeyUse, SecretName, SourceSpec};
/// Publisher policy shared by capture and the future variant section.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: biomcp_types::Source::AlphaGenome,
    terms: biomcp_fetch::Terms {
        name: "AlphaGenome API terms",
        url: "https://alphagenome.google/terms",
        usage: biomcp_types::Use::Licensed,
    },
    base: "https://gdmscience.googleapis.com",
    key: KeyUse::Required(Credential {
        tier: biomcp_types::KeyTier::NoFallback,
        secret: SecretName::AlphaGenomeApiKey,
        place: KeyPlace::Header("x-goog-api-key"),
    }),
    quota: &crate::quotas::ALPHAGENOME,
    deadline: std::time::Duration::from_secs(120),
};
/// One documented prediction method, with complete-stream admission and no cache.
pub const PREDICT_VARIANT: biomcp_fetch::grpc::GrpcEndpoint = SPEC
    .grpc(request::PREDICT_VARIANT)
    .checked(SCHEMA, biomcp_fetch::Freshness::Never);

/// Ruled legacy scorer method, admitted only after complete checked terminal success.
pub const SCORE_VARIANT: biomcp_fetch::grpc::GrpcEndpoint = SPEC
    .grpc(score::SCORE_VARIANT)
    .bounded(SCORE_LIMITS)
    .checked(SCORE_SCHEMA, biomcp_fetch::Freshness::Never);

/// Complete source policy and command declaration.
pub(crate) mod decl;
