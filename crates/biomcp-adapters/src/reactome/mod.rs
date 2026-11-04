//! Lowest-level human pathway mappings from Reactome.
use crate::{Mapped, quotas};
use biodata::UniProtAcc;
use biomcp_fetch::{Input, Read};

use biomcp_fetch::{Arg, BadReply, Freshness, KeyUse, Leg, Reply, RequestError, SourceSpec};
use biomcp_types::{PathwayRef, Source};
use serde::Deserialize;
use std::time::Duration;
mod discovery;
mod map;
mod raw;
mod relation_raw;
mod relations;
pub use discovery::{Page, card, release, search, top_level};
pub use relations::{Members, events, members};
/// Public Reactome mapping endpoint policy.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC0 1.0 database data",
        url: "https://reactome.org/license",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Reactome,
    base: "https://reactome.org/ContentService",
    key: KeyUse::None,
    quota: &quotas::REACTOME,
    deadline: Duration::from_secs(8),
};
#[derive(Deserialize)]
struct Rows(Vec<Row>);
impl Reply for Rows {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Row {
    st_id: String,
    display_name: Option<crate::raw::Field<String>>,
    species_name: String,
    schema_class: String,
}
/// Fetch one accession; a checked 404 is an empty mapping.
pub fn pathways(
    leg: &Leg<'_>,
    accession: &UniProtAcc,
) -> Result<Mapped<Vec<PathwayRef>>, RequestError> {
    let endpoint = SPEC
        .get("/data/mapping/UniProt")
        .with_path(format!("/data/mapping/UniProt/{accession}/pathways"))?
        .fresh(Freshness::Days(7));
    let rows = match leg
        .call(
            &endpoint,
            Input::Params(&[("species", Arg::Text("9606"))]),
            Read::<Rows>::json(),
        )
        .map(|received| received.value)
    {
        Ok(rows) => rows,
        Err(RequestError::Source(error)) if error.reason == biomcp_types::Reason::NotFound => {
            return Ok(Mapped {
                refusals: crate::Refusal::None,
                value: Vec::new(),
                problems: Vec::new(),
            });
        }
        Err(error) => return Err(error),
    };
    let mut value = Vec::new();
    let mut refusals = crate::Refusal::None;
    let mut problems = map::fields(accession.as_str());
    for row in rows.0 {
        let id = row.st_id.parse().ok();
        if row.species_name != "Homo sapiens" || row.schema_class != "Pathway" || id.is_none() {
            problems.required::<()>("pathways", None);
            refusals = crate::Refusal::Refused;
            continue;
        }
        if let Some(id) = id {
            let name = map::metadata(
                &mut problems,
                "pathways.name",
                row.display_name.as_ref(),
                false,
            );
            value.push(PathwayRef { id, name });
        }
    }
    Ok(Mapped {
        refusals,
        value,
        problems: problems.list,
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
