//! Encode the ruled ScoreVariant request without exposing provider protobuf types.
use super::{Limits, proto};
use super::{ScoreEnvelope, request_variant};
use biomcp_fetch::{BadReply, InputError};
use prost::Message;

/// Publisher scoring method, independent of the earlier prediction method.
pub(super) const SCORE_VARIANT: &str =
    "/google.gdm.gdmscience.alphagenome.v1main.DnaModelService/ScoreVariant";
/// Pinned scoring contract for complete-stream admission and persistent identities.
pub const SCORE_SCHEMA: &str =
    "alphagenome-55943112b83439e0c2512110f1ae607d4b5b492a-score-variant-legacy-v1";
const INTERVAL_HALF: i64 = 262_144;
/// A human GRCh38 scoring request with the released scorer order and provider-default checkpoint.
#[derive(Debug)]
pub struct ScoreRequest {
    build: biodata::Build,
    request: proto::ScoreVariantRequest,
    bytes: Vec<u8>,
}
impl ScoreRequest {
    /// Preserve the source-resolved allele and the ruled interval construction.
    /// No organism, interval, scorer or checkpoint selector is exposed.
    pub fn human_legacy(
        build: biodata::Build,
        chromosome: biodata::Chromosome,
        position: i64,
        reference: &str,
        alternate: &str,
    ) -> Result<Self, InputError> {
        // This integration has no verified mitochondrial chromosome/interval contract.
        // Do not invent chrMT/chrM normalization or make a publisher support claim.
        if chromosome.number() == 25 {
            return Err(InputError::Path);
        }
        let chromosome = format!("chr{}", chromosome.name());
        let variant = request_variant::checked(build, &chromosome, position, reference, alternate)?;
        let center = position.checked_sub(1).ok_or(InputError::Path)?;
        let start = center.saturating_sub(INTERVAL_HALF).max(0);
        let mut end = center.checked_add(INTERVAL_HALF).ok_or(InputError::Path)?;
        if start == 0 {
            end = end.max(INTERVAL_HALF * 2);
        }
        let request = proto::ScoreVariantRequest {
            interval: Some(proto::Interval {
                chromosome,
                start,
                end,
                strand: proto::Strand::Unstranded as i32,
            }),
            variant: Some(variant),
            organism: proto::Organism::HomoSapiens as i32,
            variant_scorers: scorers(),
            model_version: String::new(),
            // The released request omitted this field. Do not inherit the current SDK's true default.
            merge_stranded_gene_tracks: false,
        };
        let bytes = request.encode_to_vec();
        Ok(Self {
            build,
            request,
            bytes,
        })
    }
    /// Caller-selected assembly provenance; the wire has no build field.
    pub fn build(&self) -> biodata::Build {
        self.build
    }
    /// Stable credential-free request bytes, including ordered scorers.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Exact request interval provenance; the native score reply has no interval field.
    pub fn requested_interval(&self) -> Result<biomcp_types::AlphaGenomeInterval, BadReply> {
        let interval = self.request.interval.as_ref().ok_or(BadReply)?;
        Ok(biomcp_types::AlphaGenomeInterval {
            chromosome: interval.chromosome.parse().map_err(|_| BadReply)?,
            start: interval.start,
            end: interval.end,
            strand: biomcp_types::AlphaGenomeStrand::Unstranded,
        })
    }
    /// Start independent bounded admission for exactly the requested scorer groups.
    pub fn decoder(&self, limits: Limits) -> Result<ScoreEnvelope, BadReply> {
        ScoreEnvelope::new(
            self.request.variant.as_ref().ok_or(BadReply)?,
            self.request.variant_scorers.len(),
            limits,
        )
    }
}
fn scorers() -> Vec<proto::VariantScorer> {
    use proto::variant_scorer::Scorer;
    vec![
        proto::VariantScorer {
            scorer: Some(Scorer::GeneMask(proto::GeneMaskLfcScorer {
                requested_output: proto::OutputType::RnaSeq as i32,
            })),
        },
        proto::VariantScorer {
            scorer: Some(Scorer::GeneMaskSplicing(proto::GeneMaskSplicingScorer {
                width: None,
                requested_output: proto::OutputType::SpliceSites as i32,
            })),
        },
        proto::VariantScorer {
            scorer: Some(Scorer::CenterMask(proto::CenterMaskScorer {
                width: Some(501),
                aggregation_type: proto::AggregationType::DiffMean as i32,
                requested_output: proto::OutputType::Dnase as i32,
            })),
        },
    ]
}
