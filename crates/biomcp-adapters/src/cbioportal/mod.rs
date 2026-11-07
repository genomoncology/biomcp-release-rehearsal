//! Bounded explicit-cohort mutation observations and stored public study metadata.
use biomcp_fetch::{Input, Read};
mod commercial;
mod raw;
pub use commercial::commercial_restriction;
mod scope;
use biodata::NcbiGeneId;
use biomcp_fetch::{Arg, BadReply, Freshness, KeyUse, Leg, PathSegment, RequestError, SourceSpec};
use biomcp_types::{RecurrenceStudy, Source, VariantRecurrenceQuery};
mod map;
pub use map::Mutation;
use std::time::Duration;
/// Reviewed DataHub directory API origin.
pub(crate) const GITHUB_ORIGIN: &str = "https://api.github.com";
/// Study-specific terms must be verified independently of public read access.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Additional(&[GITHUB_ORIGIN]),
    user_agent: None,
    source: Source::Cbioportal,
    base: "https://www.cbioportal.org/api",
    key: KeyUse::None,
    quota: &crate::quotas::CBIOPORTAL,
    deadline: Duration::from_secs(15),
    terms: biomcp_fetch::Terms {
        name: "ODbL with attribution; individual studies may restrict commercial use",
        url: "https://docs.cbioportal.org/user-guide/faq/",
        usage: biomcp_types::Use::Attribution,
    },
};
/// Checked metadata for the exact caller-selected cohort.
#[derive(Debug)]
pub struct Cohort {
    /// Rejected native descriptive fields.
    pub problems: Vec<biomcp_types::Problem>,
    /// Source title, description and citation without access-group license inference.
    pub study: RecurrenceStudy,
    /// Selected list size, without assuming gene assay eligibility.
    pub selected_sample_count: Option<u64>,
    /// Failure to establish selected-list metadata; mutation observations remain usable.
    pub count_error: Option<RequestError>,
}
/// Fetch and bind all cohort selectors before reading mutation rows.
pub fn cohort(leg: &Leg<'_>, query: &VariantRecurrenceQuery) -> Result<Cohort, RequestError> {
    let endpoint = SPEC
        .get("/studies")
        .with_segments(&[PathSegment::new(query.study().as_str())?])
        .fresh(Freshness::Days(1));
    let (study, _) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            Read::<raw::Study>::json().check(&|v| {
                if v.study_id == *query.study() {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    let endpoint = SPEC
        .get("/molecular-profiles")
        .with_segments(&[PathSegment::new(query.profile().as_str())?])
        .fresh(Freshness::Days(1));
    leg.call(
        &endpoint,
        Input::Params(&[]),
        Read::<raw::Profile>::json().check(&|v| {
            if v.study_id == *query.study()
                && v.molecular_profile_id == *query.profile()
                && v.molecular_alteration_type.as_str() == "MUTATION_EXTENDED"
            {
                Ok(())
            } else {
                Err(BadReply)
            }
        }),
    )
    .map(|received| (received.value, received.meta))?;
    let mut problems = crate::mapped::Problems {
        source: Source::Cbioportal,
        record: Some(study.study_id.clone()),
        list: Vec::new(),
    };
    let name = problems.required_field("name", study.name).map(Into::into);
    let description = problems
        .field("description", study.description)
        .map(Into::into);
    let citation = problems.field("citation", study.citation).map(Into::into);
    let (selected_sample_count, count_error) = match sample_count(leg, query) {
        Ok(count) => {
            problems.list.extend(count.problems);
            (count.value, None)
        }
        Err(error) => (None, Some(error)),
    };
    Ok(Cohort {
        problems: problems.list,
        study: RecurrenceStudy {
            name,
            description,
            citation,
        },
        selected_sample_count,
        count_error,
    })
}
/// Checked mutation records with their native window and field diagnostics.
#[derive(Debug)]
pub struct MutationPage {
    /// Accepted mutations and actual refusal events.
    pub window: crate::NativeWindow<Mutation>,
    /// Independent field diagnostics.
    pub problems: Vec<biomcp_types::Problem>,
}
/// One native gene page. Wrong study, profile or gene cannot enter the cache.
pub fn mutations(
    leg: &Leg<'_>,
    query: &VariantRecurrenceQuery,
    gene: NcbiGeneId,
) -> Result<MutationPage, RequestError> {
    let endpoint = SPEC
        .get("/molecular-profiles")
        .with_segments(&[
            PathSegment::new(query.profile().as_str())?,
            PathSegment::new("mutations")?,
        ])
        .fresh(Freshness::Days(1));
    let params = [
        ("sampleListId", Arg::Text(query.sample_set().as_str())),
        ("entrezGeneId", Arg::Num(u64::from(gene.get()))),
        ("projection", Arg::Text("SUMMARY")),
        ("pageSize", Arg::Num(u64::from(query.limit()))),
        ("pageNumber", Arg::Num(u64::from(query.page()))),
    ];
    let (rows, _) = leg
        .call(
            &endpoint,
            Input::Params(&params),
            Read::<raw::Mutations>::json()
                .check(&|rows| {
                    if rows.0.len() <= query.limit() as usize
                        && rows.0.iter().all(|v| {
                            v.scope.study_id == *query.study()
                                && v.scope.molecular_profile_id == *query.profile()
                                && v.scope.entrez_gene_id == gene.get()
                        })
                    {
                        Ok(())
                    } else {
                        Err(BadReply)
                    }
                })
                .cache_if(&raw::Mutations::identities_admitted),
        )
        .map(|received| (received.value, received.meta))?;
    let received = rows.0.len() as u32;
    let mut problems = crate::mapped::Problems {
        source: Source::Cbioportal,
        record: Some(query.study().clone()),
        list: Vec::new(),
    };
    let mut values = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in rows.0 {
        if let Some(mut row) = problems.field("sampleId", Some(row.value)) {
            problems.list.append(&mut row.problems);
            values.push(map::mutation(row));
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    Ok(MutationPage {
        window: crate::NativeWindow::new(
            values,
            query.page().saturating_mul(query.limit()),
            query.limit(),
            None,
            received,
            refusals,
        ),
        problems: problems.list,
    })
}

/// Largest public list measured, 1,086,826 bytes for 54,331 members, plus fifteen percent, rounded upward.
const MAX_SAMPLE_LIST_BYTES: u64 = 1_249_850;
fn sample_count(
    leg: &Leg<'_>,
    query: &VariantRecurrenceQuery,
) -> Result<crate::Mapped<Option<u64>>, RequestError> {
    list_sample_count(leg, query.study().as_str(), query.sample_set().as_str())
}
/// The source states a list's size only on its detailed record, whose member IDs are dropped at parse.
/// A study's lists together outgrow any fixed bound on large studies, so this reads one list alone.
fn list_sample_count(
    leg: &Leg<'_>,
    study: &str,
    sample_list: &str,
) -> Result<crate::Mapped<Option<u64>>, RequestError> {
    let endpoint = SPEC
        .get("/sample-lists")
        .with_segments(&[PathSegment::new(sample_list)?])
        .fresh(Freshness::Days(1))
        .bytes(MAX_SAMPLE_LIST_BYTES);
    let (list, _) = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            Read::<raw::SampleList>::json()
                .bounded(MAX_SAMPLE_LIST_BYTES, biomcp_types::Reason::TooLarge)
                .check(&|v| {
                    if v.study_id.as_str() == study && v.sample_list_id.as_str() == sample_list {
                        Ok(())
                    } else {
                        Err(BadReply)
                    }
                }),
        )
        .map(|received| (received.value, received.meta))?;
    let mut problems = crate::mapped::Problems {
        source: Source::Cbioportal,
        record: Some(list.sample_list_id),
        list: Vec::new(),
    };
    let value = problems.field("sampleCount", list.sample_count);
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}

/// Stored public catalog metadata.
pub mod studies;
pub use studies::sync as sync_studies;

pub(crate) use studies::bundle_state;

/// Complete source policy and command declaration.
pub(crate) mod decl;
