use super::{Outcome, Request, RequestError, Selection, StepKind, complex_portal, interpro};
use biomcp_types::ReplyBody;
use biomcp_types::{ProteinComplexMembers, ProteinComplexes, ProteinDomains};
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum Saved {
    Domains(
        Vec<biodata::ProteinDomainMatch>,
        u64,
        Vec<biomcp_types::Problem>,
        bool,
    ),
    Complexes(complex_portal::Collection),
    Members(Option<complex_portal::Complex>),
}
impl Saved {
    pub(super) fn fetch(q: &Selection, leg: &biomcp_fetch::Leg<'_>) -> Result<Self, RequestError> {
        match q {
            Selection::Domains(query) => {
                let mut page = interpro::page(leg, q.accession(), None, 25)?;
                let mut cursors = std::collections::BTreeSet::new();
                while let Some(cursor) = page.cursor.take() {
                    if !cursors.insert(cursor.to_string()) {
                        return Err(biomcp_fetch::bad_reply(q.source(), Some(200)));
                    }
                    let next = interpro::page(leg, q.accession(), Some(&cursor), 25)?;
                    if next.total != page.total
                        || page.items.len().saturating_add(next.items.len()) as u64 > page.total
                        || next
                            .items
                            .iter()
                            .any(|r| page.items.iter().any(|old| old.id == r.id))
                    {
                        return Err(biomcp_fetch::bad_reply(q.source(), Some(200)));
                    }
                    page.refusals.include(next.refusals);
                    page.items.extend(next.items);
                    page.problems.extend(next.problems);
                    page.cursor = next.cursor;
                }
                if page.items.len() as u64 != page.total {
                    return Err(biomcp_fetch::bad_reply(q.source(), Some(200)));
                }
                let total = page.total;
                page.items.retain(|r| {
                    query
                        .entry_type()
                        .is_none_or(|kind| r.entry_type.as_ref() == Some(kind))
                });
                Ok(Self::Domains(
                    page.items,
                    total,
                    page.problems,
                    page.refusals == biomcp_adapters::Refusal::Refused,
                ))
            }
            Selection::Complexes(_) => {
                complex_portal::search(leg, q.accession()).map(Self::Complexes)
            }
            Selection::Members(query) => {
                complex_portal::members(leg, query.complex(), q.accession()).map(Self::Members)
            }
        }
    }
    pub(super) fn total(&self) -> u64 {
        match self {
            Self::Domains(items, _, _, _) => items.len() as u64,
            Self::Complexes(c) => c.items.len() as u64,
            Self::Members(c) => c.as_ref().map_or(0, |c| c.members.len() as u64),
        }
    }
    pub(super) fn member_steps(
        &self,
        q: &Selection,
        offset: u32,
        limit: u32,
        cx: &crate::Ctx<'_>,
        deadline: biomcp_fetch::Deadline<'_>,
        out: &mut Outcome,
    ) -> Result<(), RequestError> {
        let Self::Complexes(collection) = self else {
            return Ok(());
        };
        for complex in collection
            .items
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
        {
            let query = biomcp_types::ProteinComplexMembersQuery::new(
                q.accession().clone(),
                complex.header.id.clone(),
            );
            let selection = Selection::Members(query.clone());
            let key = selection.key()?;
            let saved = Self::Members(Some(complex.clone()));
            let pool = crate::publish::keep(
                crate::source_snapshot::intern(
                    cx,
                    &key,
                    &saved,
                    complex.members.len() as u32,
                    &out.sources,
                    deadline,
                ),
                out,
            )?;
            if let Some(pool) = pool {
                out.next.push(crate::next::Step::run(
                    StepKind::Section,
                    &[biomcp_types::Source::ComplexPortal],
                    Request::ProteinComplexMembers(query.at_page(pool.info.id, 0)),
                    false,
                ));
            }
        }
        Ok(())
    }
    pub(super) fn render(
        self,
        q: &Selection,
        offset: u32,
        limit: u32,
        out: &mut Outcome,
    ) -> Result<(), RequestError> {
        let offset = offset as usize;
        let limit = limit as usize;
        match (self, q) {
            (Self::Domains(items, native_total, problems, dropped), Selection::Domains(query)) => {
                out.dropped |= dropped;
                out.problems.extend(problems);
                out.body = (ProteinDomains {
                    items: items.into_iter().skip(offset).take(limit).collect(),
                    native_total,
                })
                .with_query(query.clone())
            }
            (Self::Complexes(c), Selection::Complexes(query)) => {
                out.include_refusals(c.refusals);
                out.problems.extend(
                    c.items
                        .iter()
                        .flat_map(|c| c.problems.clone().into_iter().flatten()),
                );
                let items: Vec<_> = c
                    .items
                    .into_iter()
                    .skip(offset)
                    .take(limit)
                    .map(|c| c.header)
                    .collect();
                out.body = (ProteinComplexes {
                    items: items.into(),
                    native_total: c.native_total,
                })
                .with_query(query.clone());
            }
            (Self::Members(Some(c)), Selection::Members(query)) => {
                out.include_refusals(c.refusals);
                out.problems.extend(c.problems.into_iter().flatten());
                let items: Vec<_> = c.members.into_iter().skip(offset).take(limit).collect();
                for member in &items {
                    if let Some(id) = &member.protein {
                        let q = biomcp_types::ProteinGet::new(
                            vec![biomcp_types::ProteinInput::Accession(id.clone())].into(),
                        )
                        .map_err(|_| biomcp_fetch::bad_reply(q.source(), Some(200)))?;
                        out.next.push(crate::next::Step::run(
                            StepKind::Pivot,
                            &[biomcp_types::Source::UniProt],
                            Request::GetProtein(q),
                            false,
                        ));
                    }
                }
                out.body = (ProteinComplexMembers {
                    items: items.into(),
                })
                .with_query(query.clone());
            }
            (Self::Members(None), Selection::Members(_)) => {}
            _ => return Err(biomcp_fetch::bad_reply(q.source(), Some(200))),
        }
        Ok(())
    }
}
