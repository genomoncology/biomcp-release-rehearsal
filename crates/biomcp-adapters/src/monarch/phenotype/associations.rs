//! Direct reverse associations share the disease row representation and mapper.
use biomcp_fetch::Input;
use biomcp_fetch::{Arg, BadReply, InputError, Leg, RequestError};
use biomcp_types::{PhenotypeGet, PhenotypeSection};
/// Read exactly one checked native page; wrong endpoints or taxa never enter cache.
pub fn get(leg: &Leg<'_>, query: &PhenotypeGet) -> Result<super::super::Page, RequestError> {
    crate::disease_sources::components::monarch(leg)?;
    let human = "NCBITaxon:9606"
        .parse::<biodata::Curie>()
        .map_err(|_| InputError::Serialization)?;
    let genes = match query.section() {
        Some(PhenotypeSection::Genes) => true,
        Some(PhenotypeSection::Diseases) => false,
        _ => return Err(InputError::Path.into()),
    };
    let mut args = vec![
        ("object", Arg::Text(query.id().as_str())),
        (
            "subject_category",
            Arg::Text(if genes {
                "biolink:Gene"
            } else {
                "biolink:Disease"
            }),
        ),
    ];
    if genes {
        args.push(("subject_taxon", Arg::Text("NCBITaxon:9606")));
    }
    args.extend([
        ("direct", Arg::Text("true")),
        ("limit", Arg::Num(query.section_limit().into())),
        ("offset", Arg::Num(query.section_offset().into())),
    ]);
    let (page, metadata) = leg
        .call(
            &super::super::ASSOCIATIONS,
            Input::Params(&args),
            biomcp_fetch::Read::<super::super::raw::Page>::json()
                .check(&|page| {
                    if page.limit != query.section_limit()
                        || page.offset != query.section_offset()
                        || page.items.iter().any(|row| {
                            row.object.as_ref().map(|id| id.as_str()) != Some(query.id().as_str())
                                || (genes && row.subject_taxon.as_ref() != Some(&human))
                        })
                    {
                        return Err(BadReply);
                    }
                    Ok(())
                })
                .cache_if(&super::super::raw::Page::identities_admitted),
        )
        .map(|received| (received.value, received.meta))?;
    let received = page.items.len() as u32;
    let mut problems = Vec::new();
    let mut items = Vec::new();
    let mut refusals = crate::Refusal::None;
    for mut row in page.items {
        problems.append(&mut row.problems);
        if row.identity_admitted() {
            let requested = row
                .object
                .clone()
                .ok_or_else(|| biomcp_fetch::bad_reply(biomcp_types::Source::Monarch, None))?;
            let mapped = super::super::map::row(row, &requested)?;
            refusals.include(mapped.refusals);
            problems.extend(mapped.problems);
            items.push(mapped.value);
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    Ok(super::super::Page {
        refusals,
        received,
        problems,
        items,
        components: super::super::components()?,
        total: page.total,
        metadata,
    })
}
