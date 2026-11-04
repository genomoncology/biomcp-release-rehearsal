//! Native query windows follow https://docs.myvariant.info/en/latest/doc/variant_query_service.html.
//! The query layer reports its result-window bound without replacing native order with scroll order.
use super::{QUERY, map, raw};
use crate::Mapped;
use biodata::Build;
use biomcp_fetch::{Arg, BadReply, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{VariantHeader, VariantSearch};
use serde::Deserialize;
const FIELDS: &str = "_id,dbsnp.rsid,dbsnp.gene.symbol,clinvar.variant_id,chrom,hg19,hg38";
/// Native MyVariant page; no local sort or deduplication changes candidate identity.
#[derive(Debug)]
pub struct SearchPage {
    /// Total reported by MyVariant for this exact query.
    pub total: u64,
    /// Native positions consumed, including refused records.
    pub consumed: u32,
    /// Candidates excluded because their reference allele could not be confirmed.
    pub excluded: u32,
    /// Failure encountered while checking an uncorroborated reference.
    pub reference_error: Option<RequestError>,
    /// Every candidate in native order, with source field problems kept separate.
    pub rows: Mapped<Vec<VariantHeader>>,
}
#[derive(Deserialize)]
struct Page {
    total: u64,
    error: Option<serde::de::IgnoredAny>,
    success: Option<bool>,
    hits: Vec<crate::raw::Field<raw::Hit>>,
}
impl Reply for Page {
    fn check(&self) -> Result<(), BadReply> {
        if self.error.is_some()
            || self.success == Some(false)
            || self.hits.len() > 50
            || self.hits.len() as u64 > self.total
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
/// Send one declared native query window and validate it before cache admission.
pub fn search(
    leg: &Leg<'_>,
    query: &VariantSearch,
    term: &str,
    reference: &Leg<'_>,
) -> Result<SearchPage, RequestError> {
    let fields = super::annotation_fields(leg, FIELDS);
    let mut args = vec![
        ("q", Arg::Text(term)),
        ("size", Arg::Num(u64::from(query.limit()))),
        ("from", Arg::Num(u64::from(query.offset()))),
        ("fields", Arg::Text(&fields)),
    ];
    if let Some(build) = query.build() {
        args.push((
            "assembly",
            Arg::Text(if build == Build::GRCh38 {
                "hg38"
            } else {
                "hg19"
            }),
        ));
    }
    let page = leg
        .call(
            &QUERY,
            Input::Params(&args),
            Read::<Page>::json()
                .check(&|page| {
                    if page.hits.len() <= query.limit() as usize
                        && page.hits.len() as u64
                            == page
                                .total
                                .saturating_sub(u64::from(query.offset()))
                                .min(u64::from(query.limit()))
                    {
                        Ok(())
                    } else {
                        Err(BadReply)
                    }
                })
                .cache_if(&|page| {
                    page.hits
                        .iter()
                        .all(|raw| raw.as_ref().is_some_and(|hit| hit.check().is_ok()))
                }),
        )?
        .value;
    Ok(mapped_page(page, leg, query, reference))
}

fn mapped_page(
    page: Page,
    leg: &Leg<'_>,
    query: &VariantSearch,
    reference: &Leg<'_>,
) -> SearchPage {
    let fallback =
        query.uses_dbnsfp_selector() && super::components::disabled_on_leg(leg, "dbnsfp");
    let mut excluded = 0;
    let mut reference_error = None;
    let mut rows = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    let consumed = page.hits.len() as u32;
    for raw in page.hits {
        let Some(hit) = raw.value() else {
            rows.refusals = crate::Refusal::Refused;
            rows.problems.push(crate::mapped::problem(
                biomcp_types::Source::MyVariant,
                None,
                "identity",
            ));
            continue;
        };
        let id = hit.id.clone();
        let build = super::search_build::source_build(&hit);
        if fallback {
            match super::corroboration::confirmed(&hit, leg, reference) {
                Ok(true) => {}
                result => {
                    if let Err(error) = result {
                        reference_error = Some(error);
                    }
                    excluded += 1;
                    continue;
                }
            }
        }
        let mapped = map::candidates(
            vec![crate::raw::Field::Value(hit)],
            id.as_str(),
            build,
            !super::components::disabled_on_leg(leg, "clinvar"),
            !super::components::disabled_on_leg(leg, "snpeff"),
        );
        for mut row in mapped.value {
            for transcript in &mut row.transcripts {
                transcript.matched = transcript.hgvs_p.as_ref().is_some_and(|p| {
                    query.matches_protein(transcript.gene.as_ref().map(|g| g.as_str()), p.as_str())
                });
            }
            rows.value.push(row);
        }
        rows.refusals.include(mapped.refusals);
        rows.problems.extend(mapped.problems);
    }
    SearchPage {
        total: page.total,
        excluded,
        reference_error,
        consumed,
        rows,
    }
}
