//! Native min and base tiers; complete bounded collections stay in the shared cache.
use super::SPEC;
use crate::{Mapped, mapped::Problems, pgx_sources::Native as NativeRow, raw::Field};
use biodata::{PgxAnnotationDetail, PgxAnnotationHeader, PgxAnnotationReference};
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{PgxAnnotationGet, PgxAnnotationKind, PgxGet};
use serde::Deserialize;
/// Bounded whole native collection with no claimed server-wide total.
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct Native<T> {
    /// Complete native result.
    data: T,
    status: Box<str>,
}
impl<T: serde::de::DeserializeOwned> Reply for Native<T> {
    fn check(&self) -> Result<(), BadReply> {
        if self.status.as_ref() == "success" {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}
fn collection<T: serde::de::DeserializeOwned + NativeRow>(
    leg: &Leg<'_>,
    q: &PgxGet,
    path: &'static str,
    key: &'static str,
    check: impl Fn(&T::Value) -> Result<(), &'static str>,
) -> Result<Mapped<Vec<T::Value>>, RequestError> {
    let endpoint = SPEC
        .get(path)
        .fresh(Freshness::Days(1))
        .bytes(4 * 1024 * 1024);
    let gene = q.gene().to_string();
    let body: Native<Vec<Field<T>>> = leg
        .call(
            &endpoint,
            Input::Params(&[(key, Arg::Text(&gene)), ("view", Arg::Text("min"))]),
            Read::json().check(&|_| Ok(())).cache_if(&|_| true),
        )?
        .value;
    let mut cx = Problems {
        source: biomcp_types::Source::ClinPgx,
        record: None,
        list: Vec::new(),
    };
    let mut value = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in body.data {
        cx.record = None;
        if let Some(raw) = cx.field("record", Some(row)) {
            cx.record = raw.record();
            let before = cx.list.len();
            if let Some(row) = raw.map(&mut cx, &mut refusals) {
                match check(&row) {
                    Ok(()) => value.push(row),
                    Err(field) if cx.list.len() == before => {
                        refusals = crate::Refusal::Refused;
                        cx.required::<()>(field, None);
                    }
                    Err(_) => refusals = crate::Refusal::Refused,
                }
            } else {
                refusals = crate::Refusal::Refused;
            }
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    Ok(Mapped {
        refusals,
        value,
        problems: cx.list,
    })
}
/// Clinical min preserves full locations, evidence labels and relationship arrays.
pub fn clinical(
    leg: &Leg<'_>,
    q: &PgxGet,
) -> Result<Mapped<Vec<PgxAnnotationHeader>>, RequestError> {
    let symbol = q.gene().to_string();
    collection::<super::annotation_raw::PgxAnnotationHeader>(
        leg,
        q,
        super::CLINICAL_PATH,
        "location.genes.symbol",
        |row: &PgxAnnotationHeader| {
            let matched = row.location.as_ref().is_some_and(|location| {
                location
                    .genes
                    .iter()
                    .any(|gene| gene.symbol.as_deref() == Some(symbol.as_str()))
            });
            if matched {
                Ok(())
            } else {
                Err("location.genes")
            }
        },
    )
}
/// Reference collections remain independent; neither supplies full guidance.
pub fn references(
    leg: &Leg<'_>,
    q: &PgxGet,
) -> Result<Mapped<Vec<PgxAnnotationReference>>, RequestError> {
    let (path, class) = match q.kind() {
        Some(PgxAnnotationKind::Guideline) => ("/data/guidelineAnnotation", "GuidelineAnnotation"),
        Some(PgxAnnotationKind::Label) => ("/data/labelAnnotation", "LabelAnnotation"),
        _ => return Err(biomcp_fetch::InputError::Serialization.into()),
    };
    collection::<super::annotation_raw::PgxAnnotationReference>(
        leg,
        q,
        path,
        "relatedGenes.symbol",
        |row| {
            if row.obj_cls.as_ref() == class {
                Ok(())
            } else {
                Err("objCls")
            }
        },
    )
}
/// Check exact returned accession on every read before cache admission.
pub fn annotation(
    leg: &Leg<'_>,
    q: &PgxAnnotationGet,
) -> Result<Mapped<Option<PgxAnnotationDetail>>, RequestError> {
    let endpoint = SPEC
        .get(super::CLINICAL_PATH)
        .with_path(format!("/data/clinicalAnnotation/{}", q.id()))?
        .fresh(Freshness::Days(1))
        .bytes(4 * 1024 * 1024);
    let body: Native<Field<super::detail_raw::PgxAnnotationDetail>> = leg
        .call(
            &endpoint,
            Input::Params(&[("view", Arg::Text("base"))]),
            Read::json().check(&|_| Ok(())).cache_if(&|_| true),
        )?
        .value;
    let mut cx = Problems {
        source: biomcp_types::Source::ClinPgx,
        record: None,
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let record = match cx.field("record", Some(body.data)) {
        Some(row) => Some(row),
        None => {
            refusals = crate::Refusal::Refused;
            None
        }
    };
    let value = record.and_then(|raw| {
        cx.record = raw.record();
        let Some(row) = raw.map(&mut cx, &mut refusals) else {
            refusals = crate::Refusal::Refused;
            return None;
        };
        if row.accession_id == *q.id() {
            Some(row)
        } else {
            refusals = crate::Refusal::Refused;
            cx.required("record", None)
        }
    });
    Ok(Mapped {
        refusals,
        value,
        problems: cx.list,
    })
}
