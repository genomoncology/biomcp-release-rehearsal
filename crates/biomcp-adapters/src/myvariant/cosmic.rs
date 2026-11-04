//! Historical fragments retain their original spellings and unverified association.
use crate::{
    Mapped,
    mapped::Problems,
    raw::{Field, OneOrMany},
};
use biodata::{Build, FiniteNumber, Text};
use biomcp_fetch::Input;

use biomcp_fetch::{Arg, BadReply, Leg, PathSegment, Reply, RequestError};
use biomcp_types::{CosmicCoordinates, CosmicIdentity, CosmicRecord, Source, VariantCosmic};
use serde::Deserialize;
/// Read one exact aggregator record, without asserting its fragment's allele identity.
pub fn get(leg: &Leg<'_>, id: &str, build: Build) -> Result<Mapped<VariantCosmic>, RequestError> {
    let endpoint = super::VARIANT
        .clone()
        .with_segments(&[PathSegment::new(id)?]);
    let assembly = if build == Build::GRCh38 {
        "hg38"
    } else {
        "hg19"
    };
    let raw = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("fields", Arg::Text("_id,cosmic")),
                ("assembly", Arg::Text(assembly)),
            ]),
            biomcp_fetch::Read::<Record>::json().cache_if(&|record| {
                record
                    .id
                    .as_ref()
                    .and_then(Field::as_ref)
                    .is_some_and(|given| given.as_str() == id)
            }),
        )?
        .value;
    let mut problems = Problems {
        source: Source::MyVariant,
        record: crate::mapped::record(id),
        list: Vec::new(),
    };
    let record_id = raw
        .id
        .and_then(Field::value)
        .filter(|given| given.as_str() == id);
    let mut refusals = crate::Refusal::None;
    let records = if let Some(record_id) = record_id {
        records(&record_id, raw.cosmic, &mut problems, &mut refusals)
    } else {
        refusals = crate::Refusal::Refused;
        problems.required::<()>("_id", None);
        Vec::new()
    };
    let components = if records.is_empty() {
        Vec::new()
    } else {
        let metadata = super::metadata::cosmic(leg)?;
        problems.list.extend(metadata.problems);
        metadata.value
    };
    Ok(Mapped {
        refusals,
        value: VariantCosmic {
            total: records.len() as u64,
            next: None,
            records: records.into(),
            components: components.into(),
        },
        problems: problems.list,
    })
}
#[derive(Deserialize)]
struct Record {
    #[serde(rename = "_id")]
    id: Option<Field<Text>>,
    cosmic: Option<Field<Rows>>,
}
impl Reply for Record {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
struct Row {
    cosmic_id: Option<Field<biodata::CosmicId>>,
    #[serde(rename = "ref")]
    reference: Option<Field<Text>>,
    alt: Option<Field<Text>>,
    chrom: Option<Field<Text>>,
    hg19: Option<Field<CosmicCoordinates>>,
    mut_nt: Option<Field<Text>>,
    mut_freq: Option<Field<FiniteNumber>>,
    tumor_site: Option<Field<OneOrMany<Field<Text>>>>,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Rows {
    Many(Vec<Field<Row>>),
    One(Row),
    InvalidContainer(serde::de::IgnoredAny),
}

fn records(
    id: &Text,
    rows: Option<Field<Rows>>,
    problems: &mut Problems,
    refusals: &mut crate::Refusal,
) -> Vec<CosmicRecord> {
    let mut records = Vec::new();
    let rows = match problems.field("cosmic", rows) {
        Some(Rows::Many(rows)) => rows,
        Some(Rows::One(row)) => vec![Field::Value(row)],
        Some(Rows::InvalidContainer(_)) => {
            problems.required::<()>("cosmic.cosmic_id", None);
            return records;
        }
        None => return records,
    };
    for raw_row in rows {
        let Some(row) = problems.field("cosmic.cosmic_id", Some(raw_row)) else {
            *refusals = crate::Refusal::Refused;
            continue;
        };
        let Some(cosmic_id) = problems.required_field("cosmic.cosmic_id", row.cosmic_id) else {
            *refusals = crate::Refusal::Refused;
            continue;
        };
        let tumor_site = problems
            .field("cosmic.tumor_site", row.tumor_site)
            .into_iter()
            .flat_map(OneOrMany::values)
            .filter_map(|raw| problems.field("cosmic.tumor_site", Some(raw)))
            .collect();
        records.push(CosmicRecord {
            myvariant_record: id.clone(),
            identity: CosmicIdentity::Unverified,
            cosmic_id,
            reference: problems.field("cosmic.ref", row.reference),
            alternate: problems.field("cosmic.alt", row.alt),
            chromosome: problems.field("cosmic.chrom", row.chrom),
            hg19: problems.field("cosmic.hg19", row.hg19),
            mut_nt: problems.field("cosmic.mut_nt", row.mut_nt),
            mut_freq: problems.field("cosmic.mut_freq", row.mut_freq),
            tumor_site,
        });
    }
    records
}
