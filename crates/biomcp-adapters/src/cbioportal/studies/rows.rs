//! Source-scoped row collections; local windows retain exact native totals.
use super::{Records, SPEC};
use crate::{mapped::Problems, raw::Field};
use biodata::{NcbiGeneId, StudyId, StudyProfileId, Text};
use biomcp_fetch::{Arg, BadReply, Freshness, Input, Leg, PathSegment, Read, Reply, RequestError};
use biomcp_types::{Source, StudyGet, StudyMutation, StudySample};
use serde::{Deserialize, Serialize};

const MAX_ROWS: u64 = 10_000_000; // cBioPortal's documented maximum page size.
const MAX_BYTES: u64 = 16 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Sample {
    study_id: StudyId,
    sample_id: Text,
    sample_type: Option<Field<Text>>,
}
#[derive(Deserialize)]
#[serde(transparent)]
struct Samples(Vec<super::super::scope::Row<Sample, super::super::scope::Study>>);
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mutation {
    sample_id: Text,
    protein_change: Option<Field<Text>>,
    mutation_type: Option<Field<Text>>,
}
#[derive(Deserialize)]
#[serde(transparent)]
struct Mutations(Vec<super::super::scope::Row<Mutation, super::super::scope::Mutation>>);
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Gene {
    entrez_gene_id: u32,
    hugo_gene_symbol: Text,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    study_id: StudyId,
    molecular_profile_id: StudyProfileId,
    molecular_alteration_type: Text,
}
#[derive(Deserialize)]
#[serde(transparent)]
struct Profiles(Vec<Profile>);
macro_rules! reply {
    ($($t:ty),+) => {$(impl Reply for $t { fn check(&self) -> Result<(), BadReply> { Ok(()) } })+};
}
reply!(Samples, Mutations, Gene, Profiles);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MutationFilter<'a> {
    entrez_gene_ids: [u32; 1],
    sample_list_id: &'a biodata::StudySampleListId,
}

fn endpoint(path: &'static str, segments: &[&str]) -> Result<biomcp_fetch::Endpoint, RequestError> {
    let segments = segments
        .iter()
        .map(|v| PathSegment::new(v))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SPEC
        .get(path)
        .with_segments(&segments)
        .fresh(Freshness::Days(1))
        .bytes(MAX_BYTES))
}
fn problems(id: &StudyId) -> Problems {
    Problems {
        source: Source::Cbioportal,
        record: crate::mapped::record(id),
        list: Vec::new(),
    }
}
fn samples_all(
    leg: &Leg<'_>,
    id: &StudyId,
) -> Result<(Samples, biomcp_fetch::ReplyMetadata), RequestError> {
    let received = leg.call(
        &endpoint("/studies", &[id.as_str(), "samples"])?,
        Input::Params(&[
            ("projection", Arg::Text("SUMMARY")),
            ("pageSize", Arg::Num(MAX_ROWS)),
            ("sortBy", Arg::Text("sampleId")),
            ("direction", Arg::Text("ASC")),
        ]),
        Read::<Samples>::json()
            .check(&|v| {
                if (v.0.len() as u64) < MAX_ROWS && v.0.iter().all(|r| r.scope.study_id == *id) {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            })
            .cache_if(&|v| v.0.iter().all(|r| r.value.as_ref().is_some())),
    )?;
    Ok((received.value, received.meta))
}
/// Sample IDs and sample types only; discarded clinical/patient attributes are never retained.
pub fn samples(leg: &Leg<'_>, q: &StudyGet) -> Result<Records<StudySample>, RequestError> {
    let (rows, meta) = samples_all(leg, q.id())?;
    let mut problems = problems(q.id());
    let total = rows.0.len();
    let rows: Vec<_> = rows
        .0
        .into_iter()
        .map(|r| {
            let row = problems.field("sampleId", Some(r.value))?;
            if row.study_id != *q.id() {
                return None;
            }
            problems.record = Some(row.sample_id.clone());
            Some(StudySample {
                sample_id: row.sample_id,
                sample_type: problems.field("sampleType", row.sample_type),
            })
        })
        .collect();
    Ok(Records {
        rejected: total - rows.iter().flatten().count(),
        window: crate::NativeWindow::local(rows, q.section_offset(), q.section_limit()),
        problems: problems.list,
        acquired: acquired(meta.fetched)?,
    })
}
/// Resolve the native gene and mutation profile, then acquire only matching source observations.
pub fn mutations(
    leg: &Leg<'_>,
    q: &StudyGet,
) -> Result<(Records<StudyMutation>, StudyProfileId, NcbiGeneId), RequestError> {
    let gene = gene(leg, q)?;
    let profile = profile(leg, q)?;
    let (sample_list_id, list_problems) = sample_list(leg, q.id())?;
    let body = MutationFilter {
        entrez_gene_ids: [gene.get()],
        sample_list_id: &sample_list_id,
    };
    let endpoint = SPEC
        .post_json("/molecular-profiles")
        .with_segments(&[
            PathSegment::new(profile.as_str())?,
            PathSegment::new("mutations")?,
            PathSegment::new("fetch")?,
        ])
        .fresh(Freshness::Days(1))
        .bytes(MAX_BYTES);
    let received = leg.call(
        &endpoint,
        Input::json(
            &[
                ("projection", Arg::Text("SUMMARY")),
                ("pageSize", Arg::Num(MAX_ROWS)),
                ("sortBy", Arg::Text("proteinChange")),
                ("direction", Arg::Text("ASC")),
            ],
            &body,
        ),
        Read::<Mutations>::json()
            .check(&|v| {
                if (v.0.len() as u64) < MAX_ROWS
                    && v.0.iter().all(|r| {
                        r.scope.study_id.as_str() == q.id().as_str()
                            && r.scope.molecular_profile_id.as_str() == profile.as_str()
                            && r.scope.entrez_gene_id == gene.get()
                    })
                {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            })
            .cache_if(&|v| v.0.iter().all(|r| r.value.as_ref().is_some())),
    )?;
    let mut problems = problems(q.id());
    problems.list.extend(list_problems);
    let total = received.value.0.len();
    let rows: Vec<_> = received
        .value
        .0
        .into_iter()
        .map(|r| {
            let row = problems.field("sampleId", Some(r.value))?;
            problems.record = Some(row.sample_id.clone());
            Some(StudyMutation {
                sample_id: row.sample_id,
                protein_change: problems.field("proteinChange", row.protein_change),
                mutation_type: problems.field("mutationType", row.mutation_type),
            })
        })
        .collect();
    Ok((
        Records {
            rejected: total - rows.iter().flatten().count(),
            window: crate::NativeWindow::local(rows, q.section_offset(), q.section_limit()),
            problems: problems.list,
            acquired: acquired(received.meta.fetched)?,
        },
        profile,
        gene,
    ))
}

fn gene(leg: &Leg<'_>, q: &StudyGet) -> Result<NcbiGeneId, RequestError> {
    let selector = q
        .gene()
        .ok_or_else(|| biomcp_fetch::bad_reply(Source::Cbioportal, None))?;
    let gene = leg
        .call(
            &endpoint("/genes", &[selector.as_str()])?,
            Input::Params(&[]),
            Read::<Gene>::json().check(&|g| {
                if g.entrez_gene_id != 0
                    && (g.hugo_gene_symbol == *selector
                        || g.entrez_gene_id.to_string() == selector.as_str())
                {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )?
        .value;
    NcbiGeneId::new(gene.entrez_gene_id)
        .map_err(|_| biomcp_fetch::bad_reply(Source::Cbioportal, None))
}
fn profile(leg: &Leg<'_>, q: &StudyGet) -> Result<StudyProfileId, RequestError> {
    let profiles = leg
        .call(
            &endpoint("/studies", &[q.id().as_str(), "molecular-profiles"])?,
            Input::Params(&[("projection", Arg::Text("SUMMARY"))]),
            Read::<Profiles>::json().check(&|p| {
                if p.0.iter().all(|p| p.study_id == *q.id()) {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )?
        .value;
    let profile = profiles
        .0
        .into_iter()
        .find(|p| {
            p.molecular_alteration_type.as_str() == "MUTATION_EXTENDED"
                && q.molecular_profile()
                    .is_none_or(|id| *id == p.molecular_profile_id)
        })
        .ok_or(biomcp_fetch::SourceError {
            source: Source::Cbioportal,
            reason: biomcp_types::Reason::Unavailable,
            status: None,
            tls_cause: None,
            retry_after: None,
        })?
        .molecular_profile_id;
    Ok(profile)
}

fn acquired(time: Option<i64>) -> Result<Option<u64>, RequestError> {
    time.map(u64::try_from)
        .transpose()
        .map_err(|_| biomcp_fetch::bad_reply(Source::Cbioportal, None))
}

fn sample_list(
    leg: &Leg<'_>,
    id: &StudyId,
) -> Result<(biodata::StudySampleListId, Vec<biomcp_types::Problem>), RequestError> {
    let lists = super::sample_lists(leg, id, 0, u32::MAX)?;
    let list = lists
        .window
        .rows
        .into_iter()
        .find(|l| {
            l.category
                .as_ref()
                .is_some_and(|c| c.as_str() == "all_cases_in_study")
        })
        .ok_or(biomcp_fetch::SourceError {
            source: Source::Cbioportal,
            reason: biomcp_types::Reason::Unavailable,
            status: None,
            tls_cause: None,
            retry_after: None,
        })?;
    Ok((list.sample_list_id, lists.problems))
}
