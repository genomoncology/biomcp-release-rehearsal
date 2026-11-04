//! Checked CIViC gene-level curation joined to HGNC Entrez identities.
use biomcp_fetch::{Input, Read};
mod claim_content;
mod claim_vocab;
mod drug;
mod drug_get;
mod drug_map;
mod drug_raw;
mod drug_request;
mod evidence_filter;
mod raw;
mod variant;
mod variant_claims;
mod variant_filter;
mod variant_map;
mod variant_raw;
mod variant_subject;
use crate::{Mapped, quotas, raw::Field};
use biodata::{GeneSymbol, NcbiGeneId, Text};

use biomcp_fetch::{Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::{CivicGene, Section, Source, SourceRecord};
pub use drug::drug_search;
pub use drug_get::drug_civic;
use std::time::Duration;
pub use variant::variant;
/// Public CIViC GraphQL source policy.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC0 1.0",
        url: "https://civicdb.org/releases/licensing",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Civic,
    base: "https://civicdb.org",
    key: KeyUse::None,
    quota: &quotas::CIVIC,
    deadline: Duration::from_secs(8),
};
const GENES: Endpoint = SPEC
    .post_json("/api/graphql")
    .idempotent()
    .fresh(Freshness::Days(7));

/// A requested identity and its independently mapped source section.
#[derive(Debug)]
pub struct GeneCivic {
    /// Submitted symbol that selected the GraphQL alias.
    pub symbol: GeneSymbol,
    /// Submitted Entrez join identity.
    pub id: NcbiGeneId,
    /// Complete section or its explicit absence/failure.
    pub civic: Mapped<Section<CivicGene>>,
}
/// Fetch up to twenty distinct symbol/Entrez pairs in one encoded aliased query.
pub fn genes(
    leg: &Leg<'_>,
    ids: &[(GeneSymbol, NcbiGeneId)],
) -> Result<Vec<GeneCivic>, RequestError> {
    let mut ids = ids.to_vec();
    ids.sort_unstable_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
    ids.dedup();
    let body = crate::graphql::Query::aliases(
        ids.iter().map(|(symbol, _)| symbol.to_string()),
        |index| {
            format!(
                "gene(entrezSymbol: $id{index}) {{ id name entrezId description link variants {{ totalCount }} }}"
            )
        },
    );
    macro_rules! fetch {
        ($($n:literal),+) => {
            match ids.len() {
                $( $n => leg.call(&GENES, Input::json(&[], &body), Read::<raw::Genes<$n>>::json()).map(|received| received.value).map(|raw| raw.data), )+
                _ => return Err(biomcp_fetch::bad_reply(Source::Civic, None)),
            }
        };
    }
    let raw = fetch!(
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20
    )?;
    Ok(ids
        .into_iter()
        .enumerate()
        .map(|(index, (symbol, id))| mapped(&raw, index, symbol, id))
        .collect())
}
fn mapped(raw: &raw::Aliases, index: usize, symbol: GeneSymbol, id: NcbiGeneId) -> GeneCivic {
    let gene = raw
        .0
        .iter()
        .find(|(name, _)| *name == format!("g{index}"))
        .and_then(|(_, gene)| gene.as_ref());
    let mut cx = crate::mapped::Problems {
        source: Source::Civic,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let value = gene
        .and_then(|gene| match map(gene, id, &mut cx) {
            Some(row) => Some(row),
            None => {
                refusals = crate::Refusal::Refused;
                None
            }
        })
        .unwrap_or(Section::Empty {
            source: Source::Civic,
            record: None,
        });
    GeneCivic {
        symbol,
        id,
        civic: Mapped {
            refusals,
            value,
            problems: cx.list,
        },
    }
}
fn map(
    gene: &raw::Gene,
    requested: NcbiGeneId,
    cx: &mut crate::mapped::Problems,
) -> Option<Section<CivicGene>> {
    let id = gene
        .id
        .as_ref()
        .and_then(Field::as_ref)
        .copied()
        .filter(|id| *id > 0 && *id <= i32::MAX as u64);
    cx.record = id.and_then(crate::mapped::record);
    let id = cx.required("id", id)?;
    let entrez = cx.optional(
        "entrezId",
        gene.entrez_id.as_ref().map(|value| {
            value
                .as_ref()
                .copied()
                .ok_or(())
                .and_then(|id| NcbiGeneId::new(id).map_err(|_| ()))
        }),
    );
    let entrez = cx.required("entrezId", entrez)?;
    if entrez != requested {
        cx.required::<()>("entrezId", None);
        return None;
    }
    let url = cx.optional(
        "link",
        gene.link.as_ref().map(|field| {
            field.as_ref().ok_or(()).and_then(|link| {
                if !link.starts_with("/features/") {
                    return Err(());
                }
                Text::new(&format!("https://civicdb.org{link}")).map_err(|_| ())
            })
        }),
    );
    let description = cx.optional(
        "description",
        gene.description.as_ref().map(|field| {
            field
                .as_ref()
                .ok_or(())
                .and_then(|text| Text::from_source(text).map_err(|_| ()))
        }),
    );
    let variants = cx.optional(
        "variants",
        gene.variants.as_ref().map(|field| field.as_ref().ok_or(())),
    );
    let variant_count = variants.and_then(|variants| {
        cx.optional(
            "variants.totalCount",
            variants
                .total_count
                .as_ref()
                .map(|field| field.as_ref().copied().ok_or(())),
        )
    });
    Some(if variant_count == Some(0) && description.is_none() {
        Section::Empty {
            source: Source::Civic,
            record: Some(SourceRecord {
                namespace: biodata::IdentifierNamespace::Unknown,
                id: cx.record.clone()?,
                url,
            }),
        }
    } else {
        Section::Data {
            source: Source::Civic,
            data: CivicGene {
                id: cx.optional("id", Some(crate::mapped::identifier("CIViC", id)))?,
                variant_count,
                url,
                description,
            },
        }
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
