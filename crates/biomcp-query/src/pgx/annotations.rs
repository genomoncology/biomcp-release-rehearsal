//! Independent ClinPGx collections share transport admission and local paging.
use super::{finish, next};
use crate::{Ctx, Outcome};
use biomcp_adapters::clinpgx;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PgxAnnotation, PgxAnnotationGet, PgxAnnotationIdentity, PgxAnnotationKind, PgxAnnotations,
    PgxGet, PgxHeader, PgxPageInfo, PgxSource, PgxTotalKind, Request, Source,
};
fn page<T>(rows: Vec<T>) -> (Box<[T]>, PgxPageInfo) {
    let total = rows.len() as u64;
    let items: Box<[T]> = rows.into();
    let page = PgxPageInfo {
        offset: 0,
        returned: items.len() as u32,
        total,
        total_kind: PgxTotalKind::Observed,
    };
    (items, page)
}
pub(super) fn body(
    q: &PgxGet,
    clinical: Vec<biodata::PgxAnnotationHeader>,
    references: Vec<biodata::PgxAnnotationReference>,
) -> PgxAnnotations {
    let header = PgxHeader {
        source: PgxSource::Clinpgx,
        gene: q.gene().clone(),
    };
    match q.kind() {
        Some(PgxAnnotationKind::Guideline) => {
            let (items, page) = page(references);
            PgxAnnotations::Guideline {
                header,
                items,
                page,
            }
        }
        Some(PgxAnnotationKind::Label) => {
            let (items, page) = page(references);
            PgxAnnotations::Label {
                header,
                items,
                page,
            }
        }
        _ => {
            let (items, page) = page(clinical);
            PgxAnnotations::Clinical {
                header,
                items,
                page,
            }
        }
    }
}
fn slice(q: &PgxGet, body: PgxAnnotations) -> PgxAnnotations {
    match body {
        PgxAnnotations::Clinical {
            header,
            items,
            page,
        } => {
            let (items, page) = local_page(q, items, page);
            PgxAnnotations::Clinical {
                header,
                items,
                page,
            }
        }
        PgxAnnotations::Guideline {
            header,
            items,
            page,
        } => {
            let (items, page) = local_page(q, items, page);
            PgxAnnotations::Guideline {
                header,
                items,
                page,
            }
        }
        PgxAnnotations::Label {
            header,
            items,
            page,
        } => {
            let (items, page) = local_page(q, items, page);
            PgxAnnotations::Label {
                header,
                items,
                page,
            }
        }
    }
}
fn local_page<T>(q: &PgxGet, items: Box<[T]>, mut page: PgxPageInfo) -> (Box<[T]>, PgxPageInfo) {
    let items: Box<[T]> = items
        .into_vec()
        .into_iter()
        .skip(q.section_offset() as usize)
        .take(q.section_limit() as usize)
        .collect();
    page.offset = q.section_offset();
    page.returned = items.len() as u32;
    (items, page)
}
pub(super) fn get(cx: &Ctx<'_>, q: &PgxGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = crate::status::empty((body(q, vec![], vec![])).with_query(q.clone()));
    match super::annotations_record::load(cx, q, deadline, &mut out) {
        Ok(Some((body, snapshot))) => {
            match snapshot.map_or_else(|| Ok(q.clone()), |t| q.with_snapshot(t)) {
                Ok(bound) => {
                    out.body = (slice(q, body)).with_query(q.clone());
                    out.next = next::steps(&Request::GetPgx(bound), &mut out);
                }
                Err(_) => out.local = Some(biomcp_fetch::InputError::Serialization.into()),
            }
        }
        Ok(None) => {}
        Err(error) => {
            out.local = crate::status::local(error);
            out.next = next::steps(&Request::GetPgx(q.clone()), &mut out);
        }
    }
    command.finish(deadline.remaining());
    out
}
pub(crate) fn annotation(cx: &Ctx<'_>, q: &PgxAnnotationGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&clinpgx::SPEC, deadline).fresh(cx.fresh);
    let mut problems = Vec::new();
    let reply = clinpgx::annotation(&leg, q).map(|mapped| {
        problems = mapped.problems;
        biomcp_types::PgxAnnotation::into_reply(
            q.clone(),
            mapped.value.map(|annotation| PgxAnnotation {
                header: PgxAnnotationIdentity {
                    source: PgxSource::Clinpgx,
                    id: q.id().clone(),
                },
                annotation,
            }),
        )
    });
    let mut out = finish(
        leg,
        reply,
        biomcp_types::PgxAnnotation::into_reply(q.clone(), None),
        Source::ClinPgx,
    );
    crate::status::append(&mut out.problems, problems);
    out.next = next::steps(&Request::GetPgxAnnotation(q.clone()), &mut out);
    command.finish(deadline.remaining());
    out
}
