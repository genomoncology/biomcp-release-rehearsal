use super::{SPEC, map::metadata, relation_raw as raw};
use crate::Mapped;
use crate::mapped::problem;
use crate::mapped::record as record_text;
use biodata::{PathwayEventRef, PathwayMember, PathwayParticipant, PathwaySourceRef, ReactomeId};
use biomcp_fetch::{Freshness, Leg, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::Source;
/// Complete source-reference membership with explicit occurrence and physical-entity counts.
#[derive(Debug)]
pub struct Members {
    /// Distinct namespaced reference rows in first occurrence order.
    pub items: Vec<PathwayMember>,
    /// Native reference occurrences before grouping.
    pub reference_occurrences: u64,
    /// Native physical entities, independent of reference and gene counts.
    pub physical_entities: u64,
}

/// Read the bounded complete physical-participant relation without interpreting display names.
pub fn members(leg: &Leg<'_>, id: &ReactomeId) -> Result<Mapped<Members>, RequestError> {
    let endpoint = SPEC
        .get("/data/participants")
        .with_path(format!("/data/participants/{id}"))?
        .bytes(8 * 1024 * 1024)
        .fresh(Freshness::Days(7));
    let rows = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            Read::<raw::Participants>::json(),
        )?
        .value;
    map_members(rows, id)
}
fn map_members(rows: raw::Participants, id: &ReactomeId) -> Result<Mapped<Members>, RequestError> {
    let invalid_reply = biomcp_fetch::bad_reply(Source::Reactome, Some(200));
    let mut problems = fields(id);
    let mut value = Members {
        items: Vec::new(),
        reference_occurrences: 0,
        physical_entities: rows.0.len() as u64,
    };
    for row in rows.0 {
        let participant = participant(
            &row,
            &mut problems,
            crate::mapped::text("Reactome").map_err(|_| invalid_reply)?,
        );
        for reference in row.ref_entities {
            value.reference_occurrences += 1;
            let (namespace, identifier) = reference.st_id.split_once(':').ok_or(invalid_reply)?;
            let name = metadata(
                &mut problems,
                "members.name",
                reference.display_name.as_ref(),
                false,
            );
            let source_url = metadata(
                &mut problems,
                "members.source_url",
                reference.url.as_ref(),
                false,
            );
            let schema_class = metadata(
                &mut problems,
                "members.schema_class",
                reference.schema_class.as_ref(),
                true,
            );
            if let Some(existing) = value.items.iter_mut().find(|m| {
                m.reference.namespace.as_deref() == Some(namespace)
                    && m.reference.id.as_str() == identifier
            }) {
                let mut participants = existing
                    .participants
                    .take()
                    .map_or_else(Vec::new, |values| values.into_vec());
                if !participants.iter().any(|p| p.db_id == participant.db_id) {
                    participants.push(participant.clone());
                }
                existing.physical_context_count = participants.len() as u64;
                existing.participants = Some(participants.into());
            } else {
                value.items.push(PathwayMember {
                    node_id: None,
                    reference: PathwaySourceRef {
                        namespace: crate::mapped::text(namespace)
                            .map_err(|_| invalid_reply)?
                            .into(),
                        id: crate::mapped::text(identifier).map_err(|_| invalid_reply)?,
                    },
                    name,
                    source_url,
                    gene: None,
                    schema_class,
                    physical_context_count: 1,
                    participants: Some(vec![participant.clone()].into()),
                });
            }
        }
    }
    Ok(Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
fn participant(
    row: &raw::Participant,
    problems: &mut crate::mapped::Problems,
    namespace: biodata::Text,
) -> PathwayParticipant {
    let name = metadata(
        problems,
        "participants.name",
        row.display_name.as_ref(),
        false,
    );
    let schema_class = metadata(
        problems,
        "participants.schema_class",
        row.schema_class.as_ref(),
        false,
    );
    PathwayParticipant {
        db_id: biodata::Identifier::new(namespace, row.pe_db_id),
        name,
        schema_class,
    }
}

/// Read contained events while retaining every unresolved native integer reference.
pub fn events(
    leg: &Leg<'_>,
    id: &ReactomeId,
) -> Result<Mapped<Vec<PathwayEventRef>>, RequestError> {
    let rows = event_rows(leg, id)?;
    let mut mapped = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    let mut problems = fields(id);
    let namespace = crate::mapped::text("Reactome")
        .map_err(|_| biomcp_fetch::bad_reply(Source::Reactome, Some(200)))?;
    for row in rows.0 {
        match row {
            raw::Event::Database(db_id) => mapped.value.push(PathwayEventRef::Database {
                db_id: biodata::Identifier::new(namespace.clone(), db_id),
            }),
            raw::Event::Object(v) => {
                if v.species_name
                    .as_deref()
                    .is_some_and(|s| s != "Homo sapiens")
                {
                    mapped.problems.push(problem(
                        Source::Reactome,
                        record_text(id.as_str()),
                        "events.species",
                    ));
                    mapped.refusals = crate::Refusal::Refused;
                    continue;
                }
                let native_id = v.st_id.as_deref().map(str::parse).transpose();
                let Ok(native_id) = native_id else {
                    mapped.problems.push(problem(
                        Source::Reactome,
                        record_text(id.as_str()),
                        "events.id",
                    ));
                    mapped.refusals = crate::Refusal::Refused;
                    continue;
                };
                mapped.value.push(PathwayEventRef::Object {
                    db_id: biodata::Identifier::new(namespace.clone(), v.db_id),
                    source_url: native_id.as_ref().and_then(|id| {
                        problems.text(
                            "events.source_url",
                            Some(&format!("https://reactome.org/content/detail/{id}")),
                        )
                    }),
                    id: native_id,
                    version: metadata(
                        &mut problems,
                        "events.version",
                        v.st_id_version.as_ref(),
                        false,
                    ),
                    name: metadata(&mut problems, "events.name", v.display_name.as_ref(), false),
                    schema_class: metadata(
                        &mut problems,
                        "events.schema_class",
                        v.schema_class.as_ref(),
                        false,
                    ),
                });
            }
        }
    }
    mapped.problems.extend(problems.list);
    Ok(mapped)
}

fn event_rows(leg: &Leg<'_>, id: &ReactomeId) -> Result<raw::Events, RequestError> {
    let endpoint = SPEC
        .get("/data/pathway/containedEvents")
        .with_path(format!("/data/pathway/{id}/containedEvents"))?
        .bytes(8 * 1024 * 1024)
        .fresh(Freshness::Days(7));
    leg.call(
        &endpoint,
        Input::Params(&[]),
        Read::<raw::Events>::json().cache_if(&|rows| {
            rows.0.iter().all(|row| match row {
                raw::Event::Database(_) => true,
                raw::Event::Object(v) => {
                    v.species_name
                        .as_deref()
                        .is_none_or(|s| s == "Homo sapiens")
                        && v.st_id
                            .as_deref()
                            .is_none_or(|id| id.parse::<ReactomeId>().is_ok())
                }
            })
        }),
    )
    .map(|received| received.value)
}

fn fields(id: &ReactomeId) -> crate::mapped::Problems {
    crate::mapped::Problems {
        source: Source::Reactome,
        record: crate::mapped::text(id.as_str()).ok(),
        list: Vec::new(),
    }
}
