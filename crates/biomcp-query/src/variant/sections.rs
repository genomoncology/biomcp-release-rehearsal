//! Candidate-specific source sections under one command deadline.
use crate::Ctx;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    AssertionSource, Problem, Reason, Section, Source, SourceOutcome, SourceState, VariantGet,
    VariantHeader, VariantSection, VariantSections,
};
pub(super) type Result<T> = (
    Section<T>,
    Vec<SourceOutcome>,
    Vec<Problem>,
    Option<RequestError>,
    bool,
);
type Completed<'a> = (VariantSections, Collected, Deadline<'a>);
pub(super) fn apply<'a>(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidates: &mut [VariantHeader],
    deadline: &mut Deadline<'a>,
    population_reason: Option<Reason>,
) -> (Vec<SourceOutcome>, Vec<Problem>, Option<RequestError>, bool) {
    let mut collected = Collected::default();
    for candidate in candidates {
        let mut sections = VariantSections::default();
        let origin = *deadline;
        let candidate_ref = &*candidate;
        let jobs: Vec<Box<dyn FnOnce() -> Completed<'a> + Send + '_>> = selected(query)
            .map(|section| {
                Box::new(move || read(cx, query, candidate_ref, origin, population_reason, section))
                    as Box<dyn FnOnce() -> Completed<'a> + Send + '_>
            })
            .collect();
        for (result, mut more, completed) in crate::fan_out::run(jobs) {
            sections.clinvar = sections.clinvar.or(result.clinvar);
            sections.erepo = sections.erepo.or(result.erepo);
            sections.population = sections.population.or(result.population);
            sections.gwas = sections.gwas.or(result.gwas);
            sections.civic = sections.civic.or(result.civic);
            sections.hotspots = sections.hotspots.or(result.hotspots);
            sections.oncokb = sections.oncokb.or(result.oncokb);
            sections.alphagenome = sections.alphagenome.or(result.alphagenome);
            sections.predict = sections.predict.or(result.predict);
            sections.conservation = sections.conservation.or(result.conservation);
            sections.cosmic = sections.cosmic.or(result.cosmic);
            collected.sources.append(&mut more.sources);
            collected.problems.append(&mut more.problems);
            collected.local = collected.local.or(more.local);
            collected.incomplete |= more.incomplete;
            if completed.remaining() < deadline.remaining() {
                *deadline = completed;
            }
        }
        if !query.sections().is_empty() {
            candidate.sections = Some(Box::new(sections));
        }
    }
    (
        collected.sources,
        collected.problems,
        collected.local,
        collected.incomplete,
    )
}
fn read<'a>(
    cx: &Ctx<'_>,
    query: &VariantGet,
    candidate: &VariantHeader,
    mut deadline: Deadline<'a>,
    population_reason: Option<Reason>,
    section: VariantSection,
) -> Completed<'a> {
    let mut sections = VariantSections::default();
    let mut collected = Collected::default();
    let candidates = std::slice::from_ref(candidate);
    macro_rules! attach {
        ($field:ident, $read:expr) => {
            sections.$field = Some(collected.keep(permitted(cx, section, || $read)))
        };
    }
    match section {
        VariantSection::Clinvar => attach!(
            clinvar,
            super::assertions::section(cx, query, candidates, deadline, AssertionSource::Clinvar)
        ),
        VariantSection::Erepo => attach!(
            erepo,
            super::assertions::section(cx, query, candidates, deadline, AssertionSource::Erepo)
        ),
        VariantSection::Predict => attach!(
            predict,
            super::predict::section(cx, query, candidates, &mut deadline)
        ),
        VariantSection::Conservation => attach!(
            conservation,
            super::conservation::section(cx, query, candidates, &mut deadline)
        ),
        VariantSection::Cosmic => attach!(
            cosmic,
            super::cosmic::section(cx, query, candidates, &mut deadline)
        ),
        VariantSection::Population => attach!(
            population,
            super::population::section(cx, query, candidates, deadline, population_reason)
        ),
        VariantSection::Gwas => {
            attach!(gwas, super::gwas::section(cx, query, candidates, deadline))
        }
        VariantSection::Civic => attach!(
            civic,
            super::civic::section(cx, query, candidates, deadline)
        ),
        VariantSection::Hotspots => attach!(
            hotspots,
            super::hotspots::section(cx, query, candidates, deadline)
        ),
        VariantSection::AlphaGenome => attach!(
            alphagenome,
            super::alphagenome::section(cx, query, candidates, deadline)
        ),
        VariantSection::Oncokb => attach!(
            oncokb,
            super::oncokb::section(cx, query, candidates, deadline)
        ),
        _ => {}
    }

    (sections, collected, deadline)
}
#[derive(Default)]
struct Collected {
    sources: Vec<SourceOutcome>,
    problems: Vec<Problem>,
    local: Option<RequestError>,
    incomplete: bool,
}
impl Collected {
    fn keep<T>(&mut self, value: Result<T>) -> Section<T> {
        retain(
            value,
            &mut self.sources,
            &mut self.problems,
            &mut self.local,
            &mut self.incomplete,
        )
    }
}
pub(super) fn source(section: VariantSection) -> Option<Source> {
    match section {
        VariantSection::Clinvar => Some(Source::ClinVar),
        VariantSection::Erepo => Some(Source::Erepo),
        VariantSection::Population => Some(Source::Gnomad),
        VariantSection::Gwas => Some(Source::Gwas),
        VariantSection::Civic => Some(Source::Civic),
        VariantSection::Oncokb => Some(Source::Oncokb),
        VariantSection::AlphaGenome => Some(Source::AlphaGenome),
        VariantSection::Hotspots => Some(Source::CancerHotspots),
        _ => None,
    }
}
fn permitted<T>(
    cx: &Ctx<'_>,
    section: VariantSection,
    read: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if let Some(source) = source(section).filter(|source| cx.fetch.disabled(*source)) {
        return (
            if matches!(section, VariantSection::Clinvar | VariantSection::Erepo) {
                Section::Skipped {
                    source,
                    reason: Reason::Disabled,
                }
            } else {
                Section::unavailable(source, Reason::Disabled)
            },
            vec![biomcp_adapters::registry::outcome(
                source,
                SourceState::Skipped,
                Some(Reason::Disabled),
            )],
            Vec::new(),
            None,
            false,
        );
    }
    read()
}
pub(super) fn retain<T>(
    value: Result<T>,
    sources: &mut Vec<SourceOutcome>,
    problems: &mut Vec<Problem>,
    local: &mut Option<RequestError>,
    incomplete: &mut bool,
) -> Section<T> {
    let (section, more, issues, error, dropped) = value;
    *incomplete |= dropped || matches!(section, Section::Unavailable { .. });
    if let Section::Unavailable {
        source,
        reason: Reason::NoJoinId,
    } = &section
    {
        problems.push(super::identity::missing(*source));
    }
    sources.extend(more);
    problems.extend(issues);
    *local = local.or(error);
    section
}

pub(super) fn selected(query: &VariantGet) -> impl Iterator<Item = VariantSection> + '_ {
    [
        VariantSection::Clinvar,
        VariantSection::Erepo,
        VariantSection::Population,
        VariantSection::Gwas,
        VariantSection::Civic,
        VariantSection::Hotspots,
        VariantSection::Oncokb,
        VariantSection::AlphaGenome,
        VariantSection::Predict,
        VariantSection::Conservation,
        VariantSection::Cosmic,
    ]
    .into_iter()
    .filter(|section| {
        query.sections().contains(section) || query.sections().contains(&VariantSection::All)
    })
}
