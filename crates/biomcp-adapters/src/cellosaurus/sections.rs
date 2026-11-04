//! Complete checked cards share one cache identity across independent section pages.
use super::{Card, raw};
use biodata::{
    CellLineGene, CellLineName, CellLineProvenance, CellLineReference, CellLineSourceId,
    CellLineVariation, CellLineXref, CellosaurusAccession,
};
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{
    CellLineCard, CellLineIdentityPage, CellLineNames, CellLineVariants, CellLineXrefs, Section,
    Source,
};
/// Read the retained complete projection with primary and qualified identity checks.
pub fn complete(
    leg: &Leg<'_>,
    accession: &CellosaurusAccession,
    source: Option<&CellLineSourceId>,
) -> Result<Card, RequestError> {
    super::fetch_card(leg, accession, source, true)
}
pub(super) fn xref(
    row: raw::Xref,
    field: &str,
    problems: &mut crate::mapped::Problems,
) -> CellLineXref {
    CellLineXref {
        namespace: row.database,
        id: row.accession,
        label: problems
            .field(&format!("{field}.label"), row.label)
            .map(Into::into),
        url: problems.field(&format!("{field}.url"), row.url),
        category: problems.field(&format!("{field}.category"), row.category),
        iri: problems.field(&format!("{field}.iri"), row.iri),
        discontinued: problems.field(&format!("{field}.discontinued"), row.discontinued),
    }
}
fn variation(
    row: raw::Variation,
    problems: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> CellLineVariation {
    let field = "sequence-variation-list";
    let (genes, references): (Vec<_>, Vec<_>) = problems
        .records("sequence-variation-list.xref-list", row.xref_list, refusals)
        .into_vec()
        .into_iter()
        .map(|r| xref(r, "sequence-variation-list.xref-list", problems))
        .partition(|r| r.namespace.as_str() == "HGNC");
    CellLineVariation {
        variation_type: problems
            .required_field(&format!("{field}.variation-type"), row.variation_type),
        mutation_type: problems.field(&format!("{field}.mutation-type"), row.mutation_type),
        description: problems
            .field(
                &format!("{field}.mutation-description"),
                row.mutation_description,
            )
            .map(Into::into),
        note: problems
            .field(&format!("{field}.variation-note"), row.variation_note)
            .map(Into::into),
        zygosity: problems.field(&format!("{field}.zygosity-type"), row.zygosity_type),
        genes: genes
            .into_iter()
            .map(|reference| CellLineGene {
                hgnc: problems.optional(
                    "sequence-variation-list.xref-list.accession",
                    Some(reference.id.as_str().parse()),
                ),
                reference,
            })
            .collect(),
        references: references.into(),
        provenance: problems
            .records(
                "sequence-variation-list.source-list",
                row.source_list,
                refusals,
            )
            .into_vec()
            .into_iter()
            .map(|r| CellLineProvenance {
                reference: problems
                    .field("sequence-variation-list.source-list.reference", r.reference)
                    .map(|r| CellLineReference {
                        resource_internal_ref: r.resource_internal_ref,
                    }),
                institution: problems
                    .field(
                        "sequence-variation-list.source-list.institution",
                        r.institution,
                    )
                    .map(Into::into),
                value: problems
                    .field("sequence-variation-list.source-list.value", r.value)
                    .map(Into::into),
                xref: problems
                    .field("sequence-variation-list.source-list.xref", r.xref)
                    .map(|r| xref(r, "sequence-variation-list.source-list.xref", problems)),
            })
            .collect(),
    }
}
pub(super) fn card(mut row: raw::Record) -> Result<crate::Mapped<CellLineCard>, RequestError> {
    let accession = row
        .accession()
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::Cellosaurus, None))?
        .clone();
    let mut problems = crate::mapped::Problems {
        source: Source::Cellosaurus,
        record: crate::mapped::record(&accession),
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let names = names(&row.name_list);
    let species = row.species_list.clone().unwrap_or_default().0.into_vec();
    let diseases = row.disease_list.clone().unwrap_or_default().0.into_vec();
    let warnings = super::map::warnings(row.comment_list.as_deref().unwrap_or_default());
    let xrefs = row
        .xref_list
        .take()
        .unwrap_or_default()
        .into_vec()
        .into_iter()
        .map(|r| xref(r, "xref-list", &mut problems))
        .collect();
    let variants = row
        .sequence_variation_list
        .take()
        .unwrap_or_default()
        .into_vec()
        .into_iter()
        .map(|r| variation(r, &mut problems, &mut refusals))
        .collect();
    let mut card = super::map::card(row)?;
    let data = |data| Section::Data {
        source: Source::Cellosaurus,
        data,
    };
    card.sections.names = Some(data(CellLineNames {
        accession: accession.clone(),
        items: super::map::capped(names, usize::MAX),
        offset: 0,
        release: None,
        fetched: None,
    }));
    card.sections.identity = Some(Section::Data {
        source: Source::Cellosaurus,
        data: CellLineIdentityPage {
            accession: accession.clone(),
            species: super::map::capped(species, usize::MAX),
            diseases: super::map::capped(diseases, usize::MAX),
            warnings: super::map::capped(warnings, usize::MAX),
            offset: 0,
            release: None,
            fetched: None,
        },
    });
    card.sections.xrefs = Some(Section::Data {
        source: Source::Cellosaurus,
        data: CellLineXrefs {
            accession: accession.clone(),
            items: super::map::capped(xrefs, usize::MAX),
            offset: 0,
            release: None,
            fetched: None,
        },
    });
    card.sections.variants = Some(Section::Data {
        source: Source::Cellosaurus,
        data: CellLineVariants {
            accession,
            items: super::map::capped(variants, usize::MAX),
            offset: 0,
            release: None,
            fetched: None,
        },
    });
    Ok(crate::Mapped {
        refusals,
        value: card,
        problems: problems.list,
    })
}

fn names(rows: &[raw::Name]) -> Vec<CellLineName> {
    rows.iter()
        .map(|r| CellLineName {
            kind: r.kind.clone(),
            value: r.value.clone(),
        })
        .collect()
}
