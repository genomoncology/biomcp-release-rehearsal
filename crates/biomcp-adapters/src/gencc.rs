//! GenCC submissions remain separate records keyed by HGNC identity.
use crate::native_text::NativeText;
use crate::{Mapped, mapped::Problems, quotas, raw::Field};
use biodata::{GeneSymbol, HgncId, Text};
use biomcp_fetch::{
    Bundle, Fetch, Find, Freshness, KeyKind, KeyUse, Keys, Leg, RequestError, Row, RowFormat,
    SourceSpec,
};
use biomcp_types::{GenccGene, GenccSubmission, Source, SyncResult};
use serde::Deserialize;
use std::{marker::PhantomData, time::Duration};

/// Public GenCC export and its publisher terms.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "CC0 1.0",
        url: "https://thegencc.org/download",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Gencc,
    base: "https://thegencc.org",
    key: KeyUse::None,
    quota: &quotas::GENCC,
    deadline: Duration::from_secs(8),
};
const SUBMISSIONS: Bundle<Submission> = Bundle {
    spec: &SPEC,
    name: "gencc-submissions",
    path: "/download/action/submissions-export-csv?format=new",
    format: RowFormat::NativeCsv {
        max_bytes: biomcp_fetch::MAX_BUNDLE_BYTES,
    },
    fresh: Freshness::Days(7),
    row: PhantomData,
};
/// Explicitly synchronize the source-owned submissions file.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<SyncResult>, RequestError> {
    let info = fetch.sync(&SUBMISSIONS)?;
    let problems = crate::mapped::bundle_problems(Source::Gencc, &info);
    Ok(crate::Mapped {
        refusals: crate::bundle_mapping::bundle_refusals(&info),
        problems,
        value: SyncResult {
            native_rows: None,
            snapshot: None,
            etag: info.etag,
            last_modified: info.last_modified,
            duplicate_identities: None,
            rows: info.rows,
            bytes: info.bytes,
            downloaded_bytes: info.downloaded_bytes,
        },
    })
}
/// Read each source submission for the checked HGNC identity.
pub fn gene(
    leg: &Leg<'_>,
    id: HgncId,
    symbol: &GeneSymbol,
) -> Result<Mapped<Option<GenccGene>>, RequestError> {
    let (mut rows, _) = leg.stored_rows(&SUBMISSIONS, Find::Exact(KeyKind::Id, &id.to_string()))?;
    let (rejected, _) = leg.stored_rows(
        &SUBMISSIONS,
        Find::Exact(KeyKind::Symbol, &rejected_key(symbol)),
    )?;
    rows.extend(rejected);
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    let submissions: Box<[_]> = rows
        .into_iter()
        .filter_map(|row| {
            let mapped = row.into_record(id);
            refusals.include(mapped.refusals);
            problems.extend(mapped.problems);
            mapped.value
        })
        .collect();
    Ok(Mapped {
        refusals,
        value: (!submissions.is_empty()).then_some(GenccGene { submissions }),
        problems,
    })
}
/// Publisher landing page; health never downloads the submissions file.
pub(crate) const HEALTH_PATH: &str = "/";
/// Inspect the stored export without a network request.
pub(crate) fn bundle_state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    leg.stored_bundle_state(&SUBMISSIONS)
}

#[derive(Deserialize)]
struct Submission {
    gene_curie: Option<Field<HgncId>>,
    gene_symbol: Option<Field<GeneSymbol>>,
    // The export identifies submissions by sgc_id and version_number.
    // https://thegencc.org/download
    sgc_id: Option<Field<Text>>,
    version_number: Option<Field<Text>>,
    disease_curie: Option<Field<Text>>,
    disease_title: Option<Field<NativeText>>,
    classification_curie: Option<Field<Text>>,
    classification_title: Option<Field<NativeText>>,
    moi_curie: Option<Field<Text>>,
    moi_title: Option<Field<NativeText>>,
    submitter_curie: Option<Field<Text>>,
    submitter_title: Option<Field<NativeText>>,
    submitted_as_date: Option<String>,
    submitted_run_date: Option<String>,
}
impl Row for Submission {
    fn keys(&self) -> Keys {
        let exact = match self.gene_curie.as_ref().and_then(Field::as_ref) {
            Some(id) => vec![(KeyKind::Id, id.to_string().into())],
            None => self
                .gene_symbol
                .as_ref()
                .and_then(Field::as_ref)
                .map(|symbol| vec![(KeyKind::Symbol, rejected_key(symbol).into())])
                .unwrap_or_default(),
        };
        Keys {
            exact,
            words: Vec::new(),
        }
    }
}
impl Submission {
    fn into_record(self, requested: HgncId) -> Mapped<Option<GenccSubmission>> {
        let mut cx = Problems {
            source: Source::Gencc,
            record: None,
            list: Vec::new(),
        };
        let id = cx.required_field("sgc_id", self.sgc_id);
        cx.record = id.clone();
        let gene = cx.required_field("gene_curie", self.gene_curie);
        if gene.is_some_and(|gene| gene != requested) {
            cx.required::<()>("gene_curie", None);
        }
        let version = cx.required_field("version_number", self.version_number);
        let mut refusals = crate::Refusal::None;
        let value = match (id, version) {
            (Some(id), Some(version)) if gene == Some(requested) => Some(GenccSubmission {
                id,
                version,
                disease_id: cx.field("disease_curie", self.disease_curie),
                disease: cx.source_field("disease_title", self.disease_title),
                classification_id: cx.field("classification_curie", self.classification_curie),
                classification: cx.source_field("classification_title", self.classification_title),
                mode_of_inheritance_id: cx.field("moi_curie", self.moi_curie),
                mode_of_inheritance: cx.source_field("moi_title", self.moi_title),
                submitter_id: cx.field("submitter_curie", self.submitter_curie),
                submitter: cx.source_field("submitter_title", self.submitter_title),
                evaluated_date: cx.text(
                    "submitted_as_date",
                    self.submitted_as_date.as_deref().filter(|v| !v.is_empty()),
                ),
                submitted_date: cx.text(
                    "submitted_run_date",
                    self.submitted_run_date.as_deref().filter(|v| !v.is_empty()),
                ),
            }),
            _ => {
                refusals = crate::Refusal::Refused;
                None
            }
        };
        Mapped {
            refusals,
            value,
            problems: cx.list,
        }
    }
}

// This key scopes a refused HGNC join by an independently checked native symbol.
fn rejected_key(symbol: &GeneSymbol) -> String {
    format!("rejected-hgnc:{symbol}")
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
