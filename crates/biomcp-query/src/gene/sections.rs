mod civic;
mod clingen;
mod constraint;
mod expression;
mod gencc;
mod oncokb;
mod pathways;
mod protein;
use crate::Ctx;
use biodata::{Gene, HgncId};

use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::GeneResolution;
use biomcp_types::{
    GeneGet, GeneItem, GeneSection, GeneSectionWord, GeneSections, Problem, Section, SourceOutcome,
    SourceState,
};
pub(super) struct SectionResult {
    pub dropped: bool,
    pub values: Vec<(HgncId, SectionValue)>,
    pub source: SourceOutcome,
    pub problems: Vec<Problem>,
    pub local: Option<RequestError>,
}
pub(super) enum SectionValue {
    Protein(Section<Box<[biomcp_types::Protein]>>),
    Civic(Section<biomcp_types::CivicGene>),
    ClinGen(Section<biomcp_types::ClinGenGene>),
    Gencc(Section<biomcp_types::GenccGene>),
    Constraint(Section<biomcp_types::Constraint>),
    Expression(Section<biomcp_types::GeneExpression>),
    Pathways(Section<biomcp_types::Capped<biomcp_types::PathwayRef>>),
    Oncokb(
        Section<biomcp_types::Oncokb>,
        Option<biomcp_types::OncokbCard>,
        bool,
    ),
}
const SECTIONS: [GeneSection; 8] = [
    GeneSection::Protein,
    GeneSection::Civic,
    GeneSection::ClinGen,
    GeneSection::Gencc,
    GeneSection::Constraint,
    GeneSection::Expression,
    GeneSection::Pathways,
    GeneSection::Oncokb,
];
pub(super) fn requested(query: &GeneGet) -> Vec<GeneSection> {
    SECTIONS
        .into_iter()
        .filter(|section| {
            query.sections().iter().any(|word| match word {
                GeneSectionWord::All => true,
                GeneSectionWord::Protein => *section == GeneSection::Protein,
                GeneSectionWord::Civic => *section == GeneSection::Civic,
                GeneSectionWord::ClinGen => *section == GeneSection::ClinGen,
                GeneSectionWord::Gencc => *section == GeneSection::Gencc,
                GeneSectionWord::Constraint => *section == GeneSection::Constraint,
                GeneSectionWord::Expression => *section == GeneSection::Expression,
                GeneSectionWord::Pathways => *section == GeneSection::Pathways,
                GeneSectionWord::Oncokb => *section == GeneSection::Oncokb,
            })
        })
        .collect()
}
fn applicable(section: GeneSection, gene: &Gene) -> bool {
    match section {
        GeneSection::Protein => protein::applicable(gene),
        GeneSection::Civic => true,
        GeneSection::ClinGen => true,
        GeneSection::Gencc => true,
        GeneSection::Constraint => constraint::applicable(gene),
        GeneSection::Expression => expression::applicable(gene),
        GeneSection::Pathways => pathways::applicable(gene),
        GeneSection::Oncokb => gene.entrez_id.is_some(),
    }
}
pub(super) fn prepare(query: &GeneGet, items: &mut [GeneItem]) -> Vec<(HgncId, Gene)> {
    let requested = requested(query);
    let mut genes = Vec::new();
    for item in items {
        let Some(GeneResolution::Resolved { gene, .. }) = &item.answer else {
            continue;
        };
        item.more = SECTIONS
            .into_iter()
            .filter(|section| {
                !requested.contains(section)
                    && applicable(*section, gene)
                    && *section != GeneSection::Oncokb
            })
            .collect();
        if !requested.is_empty() {
            item.sections = Some(GeneSections {
                protein: requested
                    .contains(&GeneSection::Protein)
                    .then_some(Section::NotApplicable),
                civic: requested
                    .contains(&GeneSection::Civic)
                    .then_some(Section::Empty {
                        source: GeneSection::Civic.source(),
                        record: None,
                    }),
                gencc: requested
                    .contains(&GeneSection::Gencc)
                    .then_some(Section::NotApplicable),
                clingen: requested
                    .contains(&GeneSection::ClinGen)
                    .then_some(Section::NotApplicable),
                constraint: requested
                    .contains(&GeneSection::Constraint)
                    .then_some(Section::NotApplicable),
                expression: requested
                    .contains(&GeneSection::Expression)
                    .then_some(Section::NotApplicable),
                pathways: requested
                    .contains(&GeneSection::Pathways)
                    .then_some(Section::NotApplicable),
                oncokb: requested
                    .contains(&GeneSection::Oncokb)
                    .then_some(Section::NotApplicable),
            });
        }
        if !genes.iter().any(|(id, _)| *id == gene.hgnc_id) {
            genes.push((gene.hgnc_id, gene.as_ref().clone()));
        }
    }
    genes
}
pub(super) fn offer_oncokb(
    query: &GeneGet,
    items: &mut [GeneItem],
    sources: &[biomcp_types::SourceOutcome],
) {
    if requested(query).contains(&GeneSection::Oncokb)
        || !sources.iter().any(|source| {
            source.source == biomcp_types::Source::Oncokb
                && !matches!(
                    source.reason,
                    Some(biomcp_types::Reason::NoKey | biomcp_types::Reason::Disabled)
                )
        })
    {
        return;
    }
    for item in items {
        if matches!(&item.answer, Some(GeneResolution::Resolved { gene, .. }) if applicable(GeneSection::Oncokb, gene))
        {
            item.more = item
                .more
                .iter()
                .copied()
                .chain([GeneSection::Oncokb])
                .collect();
        }
    }
}
pub(super) fn jobs<'a>(
    cx: &'a Ctx<'a>,
    deadline: Deadline<'a>,
    query: &GeneGet,
    genes: &'a [(HgncId, Gene)],
) -> Vec<Box<dyn FnOnce() -> super::get::Job + Send + 'a>> {
    let mut jobs: Vec<Box<dyn FnOnce() -> super::get::Job + Send + 'a>> = Vec::new();
    let requested = requested(query);
    if requested.contains(&GeneSection::Protein)
        && genes
            .iter()
            .any(|(_, gene)| applicable(GeneSection::Protein, gene))
    {
        jobs.push(Box::new(move || {
            super::get::Job::Section(protein::run(cx, deadline, genes))
        }));
    }
    if requested.contains(&GeneSection::Civic)
        && genes
            .iter()
            .any(|(_, gene)| applicable(GeneSection::Civic, gene))
    {
        jobs.push(Box::new(move || {
            super::get::Job::Section(civic::run(cx, deadline, genes))
        }));
    }
    if requested.contains(&GeneSection::ClinGen)
        && genes
            .iter()
            .any(|(_, gene)| applicable(GeneSection::ClinGen, gene))
    {
        jobs.push(Box::new(move || {
            super::get::Job::Section(clingen::run(cx, deadline, genes))
        }));
    }
    if requested.contains(&GeneSection::Gencc) && !genes.is_empty() {
        jobs.push(Box::new(move || {
            super::get::Job::Section(gencc::run(cx, deadline, genes))
        }));
    }
    if requested.contains(&GeneSection::Constraint)
        && genes
            .iter()
            .any(|(_, gene)| applicable(GeneSection::Constraint, gene))
    {
        jobs.push(Box::new(move || {
            super::get::Job::Section(constraint::run(cx, deadline, genes))
        }));
    }
    if requested.contains(&GeneSection::Expression)
        && genes
            .iter()
            .any(|(_, gene)| applicable(GeneSection::Expression, gene))
    {
        jobs.push(Box::new(move || {
            super::get::Job::Section(expression::run(cx, deadline, genes))
        }));
    }
    if requested.contains(&GeneSection::Pathways)
        && genes
            .iter()
            .any(|(_, gene)| applicable(GeneSection::Pathways, gene))
    {
        jobs.push(Box::new(move || {
            super::get::Job::Section(pathways::run(cx, deadline, genes))
        }));
    }
    if !genes.is_empty() {
        jobs.push(Box::new(move || {
            super::get::Job::Section(oncokb::run(cx, deadline, genes))
        }));
    }
    jobs
}
impl SectionValue {
    fn unavailable(&self) -> bool {
        match self {
            Self::Protein(value) => matches!(value, Section::Unavailable { .. }),
            Self::Civic(value) => matches!(value, Section::Unavailable { .. }),
            Self::ClinGen(Section::Data { data, .. }) => {
                [&data.validity_status, &data.dosage_status]
                    .into_iter()
                    .any(|outcome| {
                        matches!(
                            outcome.state,
                            SourceState::Failed | SourceState::Stale | SourceState::Syncing
                        )
                    })
            }
            Self::ClinGen(value) => matches!(value, Section::Unavailable { .. }),
            Self::Gencc(value) => matches!(value, Section::Unavailable { .. }),
            Self::Constraint(value) => matches!(value, Section::Unavailable { .. }),
            Self::Expression(value) => matches!(value, Section::Unavailable { .. }),
            Self::Pathways(value) => matches!(value, Section::Unavailable { .. }),
            Self::Oncokb(value, _, _) => matches!(value, Section::Unavailable { .. }),
        }
    }
    fn apply(&self, item: &mut GeneItem) {
        if let Self::Oncokb(_, card, partial) = self {
            item.oncokb = card.clone();
            if *partial {
                item.status = crate::status::item_status(true, false, true);
            }
        }
        if let Some(sections) = &mut item.sections {
            match self {
                Self::Protein(value) => sections.protein = Some(value.clone()),
                Self::Civic(value) => sections.civic = Some(value.clone()),
                Self::ClinGen(value) => sections.clingen = Some(value.clone()),
                Self::Gencc(value) => sections.gencc = Some(value.clone()),
                Self::Constraint(value) => sections.constraint = Some(value.clone()),
                Self::Expression(value) => sections.expression = Some(value.clone()),
                Self::Pathways(value) => sections.pathways = Some(value.clone()),
                Self::Oncokb(value, _, _) if sections.oncokb.is_some() => {
                    sections.oncokb = Some(value.clone())
                }
                Self::Oncokb(..) => {}
            }
        }
    }
}
pub(super) fn join(items: &mut [GeneItem], result: &SectionResult) {
    for item in items {
        let Some(GeneResolution::Resolved { gene, .. }) = &item.answer else {
            continue;
        };
        let Some((_, value)) = result.values.iter().find(|(id, _)| *id == gene.hgnc_id) else {
            continue;
        };
        if !matches!(value, SectionValue::Oncokb(..))
            && (value.unavailable()
                || (result.source.state == SourceState::Stale
                    && !matches!(value, SectionValue::ClinGen(_)))
                || result.dropped)
        {
            item.status = crate::status::item_status(true, false, true);
        }
        value.apply(item);
    }
}
