//! Native transcript/residue counts; public reachability grants no redistribution rights.
use crate::{Mapped, mapped::Problems, quotas, raw::Field};
use biodata::{GeneSymbol, Text};

use biomcp_fetch::{
    BadReply, Endpoint, Freshness, KeyUse, Leg, PathSegment, Reply, RequestError, SourceSpec,
};
use biomcp_types::{HotspotAlternate, HotspotMembership, HotspotRecord, Source};
use serde::Deserialize;
use std::time::Duration;
/// ODbL attribution and share-alike apply to database reuse.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "ODbL: attribution and share-alike",
        url: "https://www.cancerhotspots.org/",
        usage: biomcp_types::Use::Attribution,
    },
    source: Source::CancerHotspots,
    base: "https://www.cancerhotspots.org",
    key: KeyUse::None,
    quota: &quotas::HOTSPOTS,
    deadline: Duration::from_secs(8),
};
const GENE: Endpoint = SPEC
    .get("/api/hotspots/single/byGene")
    .fresh(Freshness::Days(7));
#[derive(Deserialize)]
#[serde(transparent)]
struct Rows(Vec<Field<Row>>);
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    // The publisher identifies a candidate by gene and residue; transcript is optional.
    // https://github.com/cBioPortal/cancerhotspots/blob/master/model/src/main/java/org/cmo/cancerhotspots/model/HotspotMutation.java
    hugo_symbol: Option<Field<GeneSymbol>>,
    transcript_id: Option<Field<Text>>,
    residue: Option<Field<Text>>,
    tumor_count: Option<Field<u64>>,
    variant_amino_acid: Option<Field<std::collections::BTreeMap<String, Field<u64>>>>,
}
impl Reply for Rows {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
/// Read native gene rows as candidates until a transcript/consequence join is established.
pub fn gene(leg: &Leg<'_>, gene: &GeneSymbol) -> Result<Mapped<Vec<HotspotRecord>>, RequestError> {
    let segment = PathSegment::new(gene.as_str()).map_err(RequestError::Input)?;
    let endpoint = GENE.clone().with_segments(&[segment]);
    let raw: Rows = leg
        .call(
            &endpoint,
            biomcp_fetch::Input::Params(&[]),
            biomcp_fetch::Read::json(),
        )?
        .value;
    let mut out = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    for row in raw.0 {
        let mut cx = Problems {
            source: Source::CancerHotspots,
            record: None,
            list: Vec::new(),
        };
        let mapped = cx
            .field("record", Some(row))
            .and_then(|row| row.map(gene, &mut cx));
        match mapped {
            Some(row) => out.value.push(row),
            None => out.refusals = crate::Refusal::Refused,
        }
        out.problems.extend(cx.list);
    }
    Ok(out)
}
impl Row {
    fn map(self, gene: &GeneSymbol, cx: &mut Problems) -> Option<HotspotRecord> {
        let residue = cx.required_field("residue", self.residue);
        cx.record = residue.clone();
        let symbol = cx.required_field("hugoSymbol", self.hugo_symbol);
        let (Some(residue), Some(symbol)) = (residue, symbol) else {
            return None;
        };
        if &symbol != gene {
            cx.required::<()>("hugoSymbol", None);
            return None;
        }
        let alternates = cx
            .field("variantAminoAcid", self.variant_amino_acid)
            .into_iter()
            .flatten()
            .filter_map(|(key, count)| {
                let amino_acid = cx.text("variantAminoAcid.amino_acid", Some(&key))?;
                Some(HotspotAlternate {
                    amino_acid,
                    count: cx.field("variantAminoAcid.count", Some(count)),
                })
            })
            .collect();
        Some(HotspotRecord {
            membership: HotspotMembership::Candidate,
            gene: symbol,
            transcript: cx.field("transcriptId", self.transcript_id),
            residue,
            tumor_count: cx.field("tumorCount", self.tumor_count),
            alternates,
        })
    }
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
