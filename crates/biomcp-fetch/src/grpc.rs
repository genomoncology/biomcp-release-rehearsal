//! Execute bounded streaming RPCs with checked admission and recording.
pub(super) const FRAME_BYTES: usize = tonic::codec::HEADER_SIZE;
mod bounds;
mod checked;
mod persistence;
pub(crate) mod spool;
pub use checked::StreamDecoder;
mod connect;
mod control;
mod dns;
mod h2_service;
mod raw_codec;
mod rpc;
mod rpc_call;
mod worker;
pub use rpc::GrpcEndpoint;
pub(crate) use worker::Worker;
mod decompress;
pub use decompress::decompress;

pub use bounds::Limits;

/// A codec failure never contains provider bytes or metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecError {
    /// The supplied bounds cannot describe a finite supported allocation.
    Limits,
    /// The message violates the schema, ordering or configured resource bounds.
    BadReply,
}
impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Limits => "grpc.limits",
            Self::BadReply => "grpc.bad_reply",
        })
    }
}
impl std::error::Error for CodecError {}

#[cfg(test)]
#[path = "../tests/grpc_worker/shutdown.rs"]
mod shutdown_tests;
