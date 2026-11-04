use super::{
    CellLineGet, Ctx, Outcome, Request, Source, Status, StepKind, cellosaurus, empty, recovery,
    run, status,
};
use biomcp_types::ReplyBody;
use biomcp_types::ReplyData;
use biomcp_types::{CellLineCard, CellLineInput, CellLineSection, CellLineSections};
pub(super) fn get(cx: &Ctx<'_>, query: &CellLineGet) -> Outcome {
    if query.section() == Some(CellLineSection::Chembl)
        && let CellLineInput::Accession(accession) = query.id()
    {
        return super::chembl::get(cx, query, accession);
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&cellosaurus::SPEC, deadline).fresh(cx.fresh);
    let mut card = vacant(query);
    let mut problems = Vec::new();
    let reply = resolve(&leg, query, &mut card, &mut problems);
    let error = reply.as_ref().err().copied();
    let mut out = empty((vacant(query)).with_query(query.clone()));
    out.problems = problems;
    out.sources
        .push(status::source(Source::Cellosaurus, leg.finish(), error));
    out.local = error.and_then(status::local);
    if reply.is_ok() {
        let state = card.status;
        crate::status::restored(&mut out, state);
        if card
            .sex
            .as_ref()
            .is_some_and(biodata::CellLineSex::is_unknown)
        {
            out.problems.push(biomcp_types::Problem {
                source: Source::Cellosaurus,
                record: card
                    .header
                    .as_ref()
                    .and_then(|h| biodata::Text::new(h.accession.as_str()).ok()),
                field: "sex".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
        }
        if query.section() == Some(CellLineSection::Chembl)
            && let Some(header) = &card.header
        {
            let mut metadata = super::chembl::get_at(cx, query, &header.accession, deadline);
            metadata.sources.splice(0..0, out.sources);
            if metadata.status() != Status::Failed
                && metadata
                    .sources
                    .iter()
                    .any(|source| source.state == biomcp_types::SourceState::Stale)
            {
                crate::status::item(&mut metadata, true, false, true);
            }
            command.finish(deadline.remaining());
            return metadata;
        }
        if query.section().is_some()
            && query.section() != Some(CellLineSection::All)
            && card.candidates.is_empty()
        {
            super::sections::actions(query, &card, &mut out);
        } else {
            actions(&mut out, &card);
        }
        out.body = card.with_query(query.clone());
        super::provenance::attach(cx, deadline, &mut out);
        if let ReplyData::CellLineCard {
            body: Some(card), ..
        } = &mut out.body
        {
            super::sections::provenance(card);
        }
    }
    if query.section() == Some(CellLineSection::All) && out.status() != Status::Failed {
        super::all::attach(cx, deadline, &mut out);
    }
    recovery(&mut out, Request::GetCellLine(query.clone()));
    command.finish(deadline.remaining());
    out
}
fn resolve(
    leg: &biomcp_fetch::Leg<'_>,
    query: &CellLineGet,
    card: &mut CellLineCard,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Result<(), biomcp_fetch::RequestError> {
    let mut dropped = false;
    let (accession, source) = match query.id() {
        CellLineInput::Accession(id) => (id.clone(), None),
        CellLineInput::Source(id) => {
            if cellosaurus::native_database(id).is_none() {
                return Err(biomcp_fetch::InputError::Path.into());
            }
            let page = cellosaurus::reverse(leg, id)?;
            dropped |= page.refusals == biomcp_adapters::Refusal::Refused;
            problems.extend(page.problems);
            card.fetched = page
                .metadata
                .fetched
                .and_then(|value| u64::try_from(value).ok());
            if page.items.len() != 1 {
                card.status = if page.items.is_empty() {
                    crate::status::item_status(false, false, false)
                } else {
                    crate::status::item_status(
                        true,
                        false,
                        page.refusals == biomcp_adapters::Refusal::Refused,
                    )
                };
                card.candidates = page.items.into();
                return Ok(());
            }
            (
                page.items
                    .first()
                    .ok_or(biomcp_fetch::InputError::Serialization)?
                    .accession
                    .clone(),
                Some(id),
            )
        }
    };
    let reply = if query.section().is_some() {
        cellosaurus::complete(leg, &accession, source)?
    } else {
        cellosaurus::get(leg, &accession, source)?
    };
    dropped |= reply.refusals == biomcp_adapters::Refusal::Refused;
    problems.extend(reply.problems);
    if let Some(mut selected) = reply.card {
        selected.input = query.id().clone();
        selected.status = status::item_status(true, false, dropped);
        selected.fetched = reply
            .metadata
            .fetched
            .and_then(|value| u64::try_from(value).ok());
        let overflow = selected
            .header
            .as_ref()
            .is_some_and(|h| h.species.total > 3 || h.diseases.total > 3)
            || selected.warnings.as_ref().is_some_and(|w| w.total > 3);
        if !overflow {
            selected.more = selected
                .more
                .into_vec()
                .into_iter()
                .filter(|s| *s != CellLineSection::Identity)
                .collect();
        }
        super::sections::select(query, &mut selected);
        *card = selected;
    } else {
        card.status = crate::status::item_status(false, false, false);
        card.fetched = reply
            .metadata
            .fetched
            .and_then(|value| u64::try_from(value).ok());
    }
    Ok(())
}
fn actions(out: &mut Outcome, card: &CellLineCard) {
    for candidate in &card.candidates {
        out.next.push(crate::next::Step::new(
            StepKind::Disambiguate,
            &[biomcp_types::Source::Cellosaurus],
            run(Request::GetCellLine(CellLineGet::accession(
                candidate.accession.clone(),
            ))),
        ));
    }
    if let Some(header) = &card.header {
        for section in card.more.iter().take(5) {
            out.next.push(crate::next::Step::new(
                StepKind::Section,
                &[super::sections::section_source(Some(*section))],
                run(Request::GetCellLine(
                    CellLineGet::accession(header.accession.clone()).with_section(*section),
                )),
            ));
        }
    }
}
fn vacant(query: &CellLineGet) -> CellLineCard {
    CellLineCard {
        input: query.id().clone(),
        status: crate::status::item_status(false, true, false),
        header: None,
        candidates: Box::default(),
        dates: None,
        age: None,
        sex: None,
        warnings: None,
        url: None,
        sections: CellLineSections {
            names: None,
            identity: None,
            xrefs: None,
            variants: None,
            chembl: None,
        },
        more: Box::default(),
        release: None,
        fetched: None,
    }
}
