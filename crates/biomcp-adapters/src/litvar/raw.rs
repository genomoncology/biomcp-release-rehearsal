//! Admit native candidate and publication identities without discarding optional labels.
use crate::{Mapped, mapped::Problems, raw::Field};
use biodata::Text;
use biomcp_fetch::{BadReply, PathSegment, Reply};
use biomcp_types::Source;
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Candidates(Vec<Field<Candidate>>);
#[derive(Deserialize)]
struct Candidate {
    #[serde(rename = "_id", alias = "id")]
    id: String,
    rsid: Option<Field<Text>>,
    gene: Option<Field<Vec<Field<Text>>>>,
    name: Option<Field<crate::native_text::NativeText>>,
    pmids_count: Option<Field<u64>>,
}
impl Candidate {
    fn admitted(&self) -> bool {
        PathSegment::new(&self.id).is_ok()
    }
}
impl Candidates {
    pub(super) fn eligible(&self) -> bool {
        self.0
            .iter()
            .all(|row| row.as_ref().is_some_and(Candidate::admitted))
    }
    pub(super) fn mapped(self) -> Mapped<Vec<super::Candidate>> {
        let mut problems = Problems {
            source: Source::LitVar,
            record: None,
            list: Vec::new(),
        };
        let mut value = Vec::new();
        let mut refusals = crate::Refusal::None;
        for row in self.0 {
            problems.record = None;
            let Some(row) = problems.field("identity", Some(row)) else {
                refusals = crate::Refusal::Refused;
                continue;
            };
            problems.record = crate::mapped::record(&row.id);
            if !row.admitted() {
                refusals = crate::Refusal::Refused;
                problems.required::<()>("identity", None);
                continue;
            }
            value.push(super::Candidate {
                id: row.id,
                rsid: problems.field("rsid", row.rsid).and_then(|value| {
                    problems.optional(
                        "rsid",
                        Some(if super::is_rsid(value.as_str()) {
                            Ok(value)
                        } else {
                            Err(BadReply)
                        }),
                    )
                }),
                gene: problems.fields("gene", row.gene).into_vec(),
                name: problems.source_field("name", row.name),
                pmids_count: problems.field("pmids_count", row.pmids_count),
            });
        }
        Mapped {
            refusals,
            value,
            problems: problems.list,
        }
    }
}
impl Reply for Candidates {
    fn check(&self) -> Result<(), BadReply> {
        if self.0.len() > 10_000 {
            return Err(BadReply);
        }
        Ok(())
    }
}
#[derive(Deserialize)]
pub(super) struct Publications {
    pub(super) pmids: Vec<Field<u32>>,
    pub(super) pmids_count: Option<Field<u64>>,
}
impl Publications {
    pub(super) fn eligible(&self) -> bool {
        self.pmids
            .iter()
            .all(|id| id.as_ref().is_some_and(|id| *id > 0))
    }
}
impl Reply for Publications {
    fn discovery(&self) -> Option<biomcp_fetch::Identities> {
        biomcp_fetch::Identities::bounded(
            "pmid",
            self.pmids
                .iter()
                .filter_map(Field::as_ref)
                .map(ToString::to_string)
                .collect(),
        )
    }
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
