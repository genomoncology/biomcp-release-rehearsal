//! Encode the reviewed human DNASE request without exposing generated provider types.

use super::{Limits, proto};
use biomcp_fetch::{BadReply, InputError};

use prost::Message;

/// Pinned publisher RPC path; the API version is not a model release.
pub(super) const PREDICT_VARIANT: &str =
    "/google.gdm.gdmscience.alphagenome.v1main.DnaModelService/PredictVariant";
/// Pinned protobuf identity used by checked cache and recording admission.
pub const SCHEMA: &str =
    "alphagenome-55943112b83439e0c2512110f1ae607d4b5b492a-predict-variant-dnase";

/// A validated explicit request, with one-based variant and zero-based half-open interval.
/// This request selects human DNASE, UBERON:0002048 (lung) and the ALL_FOLDS checkpoint.
#[derive(Debug)]
pub struct DnaseRequest {
    build: biodata::Build,
    request: proto::PredictVariantRequest,
    bytes: Vec<u8>,
}
impl DnaseRequest {
    /// Encode the selected request shape. Coordinates and alleles are supplied unchanged.
    /// Only the reviewed GRCh38 assembly is admitted. The publisher has no build request field.
    pub fn human_lung_all_folds(
        build: biodata::Build,
        chromosome: &str,
        interval: std::ops::Range<i64>,
        position: i64,
        reference: &str,
        alternate: &str,
    ) -> Result<Self, InputError> {
        let width = interval
            .end
            .checked_sub(interval.start)
            .ok_or(InputError::Path)?;
        let variant =
            super::request_variant::checked(build, chromosome, position, reference, alternate)?;
        let reference_len = i64::try_from(reference.len()).map_err(|_| InputError::BodyLimit)?;
        let variant_end = position
            .checked_sub(1)
            .and_then(|p| p.checked_add(reference_len));
        if interval.start < 0
            || width != 16384
            || position <= interval.start
            || variant_end.is_none_or(|end| end > interval.end)
        {
            return Err(InputError::Path);
        }
        let request = proto::PredictVariantRequest {
            interval: Some(proto::Interval {
                chromosome: chromosome.into(),
                start: interval.start,
                end: interval.end,
                strand: proto::Strand::Unstranded as i32,
            }),
            variant: Some(variant),
            organism: proto::Organism::HomoSapiens as i32,
            requested_outputs: vec![proto::OutputType::Dnase as i32],
            ontology_terms: vec![proto::OntologyTerm {
                ontology_type: proto::OntologyType::Uberon as i32,
                id: 2048,
            }],
            model_version: "ALL_FOLDS".into(),
        };
        let bytes = request.encode_to_vec();
        Ok(Self {
            build,
            request,
            bytes,
        })
    }
    /// Caller-selected assembly provenance; this is not a provider-echoed field.
    pub fn build(&self) -> biodata::Build {
        self.build
    }
    /// Stable credential-free wire bytes, including all selectors and checkpoint identity.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Start an independent bounded validator for this request's response stream.
    pub fn decoder(&self, limits: Limits) -> Result<super::Envelope, BadReply> {
        super::Envelope::new(self.request.interval.as_ref().ok_or(BadReply)?, limits)
    }
}
