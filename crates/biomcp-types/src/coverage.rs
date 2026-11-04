//! Source coverage for public command discovery, including implicit source sections.
use crate::Source;

/// Sources not fully represented by a command source selector.
pub fn command_implicit_sources(verb: &str, entity: &str) -> &'static [Source] {
    use Source::{
        AlleleRegistry, AlphaGenome, CancerHotspots, Cbioportal, Cellosaurus, Chembl, Civic,
        ClinGen, ClinVar, ClinicalTrialsGov, ComplexPortal, Erepo, EuropePmc, Gencc, Gnomad, Gwas,
        Hgnc, Hpa, Icite, InterPro, Mutalyzer, MyGene, MyVariant, Oncokb, Orcid, PmcOa, PubMed,
        PubTator, Reactome, SemanticScholar, UniProt, VariantValidator, WikiPathways,
    };
    match (verb, entity) {
        ("protein", "domains") => &[InterPro],
        ("protein", "complexes" | "complex-members") => &[ComplexPortal],
        ("protein", _) | ("search", "protein") => &[UniProt],
        ("get", "protein") => &[Hgnc, UniProt, InterPro, ComplexPortal],
        ("cell-line", _) => &[Chembl],
        ("search" | "get", "cell-line") => &[Cellosaurus],
        ("pathway", "events") => &[Reactome],
        ("pathway", _) | ("search" | "get", "pathway") => &[Reactome, WikiPathways],
        ("variant", "assertions") => &[ClinVar, Erepo],
        ("variant", "population") => &[Gnomad, MyVariant, AlleleRegistry],
        ("variant", "recurrence") => &[Cbioportal],
        ("variant", "normalize") => &[Mutalyzer, VariantValidator],
        ("get", "variant") => &[
            MyVariant,
            ClinVar,
            Erepo,
            Gnomad,
            Gwas,
            Civic,
            CancerHotspots,
            AlphaGenome,
            Oncokb,
            AlleleRegistry,
        ],
        ("search", "gene") => &[Hgnc],
        ("get", "gene") => &[
            Hgnc, MyGene, UniProt, Civic, ClinGen, Gencc, Gnomad, Hpa, Reactome, Oncokb,
        ],
        ("article", "text") => &[PubMed, PmcOa, EuropePmc],
        ("article", "supplement") => &[PubMed, PmcOa],
        ("article", "pdf") => &[SemanticScholar],
        ("article", "citation-evidence") => &[SemanticScholar, PubMed, PmcOa],
        ("article", "recommendations") => &[SemanticScholar],
        ("get", "article") => &[PubMed, EuropePmc, PmcOa, Icite, PubTator, SemanticScholar],
        ("article", _) => &[PubMed],
        ("trial", _) => &[ClinicalTrialsGov],
        ("author", _) | ("get", "author") => &[SemanticScholar, Orcid],
        (_, "study") => &[Cbioportal],
        (_, "gwas") => &[Gwas],
        _ => &[],
    }
}

/// Section publishers from their checked type owners; no renderer owns these bindings.
/// None leaves single-source command discovery to the command declaration.
pub fn command_section_sources(
    verb: &str,
    entity: &str,
    word: &str,
    candidates: &[Source],
) -> Option<Vec<Source>> {
    use crate::{
        AdverseEventSection, ArticleSection, AuthorPaperSection, CellLineSection,
        DiagnosticGetSource, DiagnosticSection, DiseaseSection, DrugSection, DrugSource,
        GeneSection, PathwaySection, PgxSection, PhenotypeSection, ProteinSection, TrialSection,
        TrialSource, VaersSection, VariantSection,
    };
    match (verb, entity) {
        ("author", "papers") => many(word, AuthorPaperSection::sources),
        ("get", "gene") => one(word, GeneSection::source),
        ("get", "variant") => one(word, |s: VariantSection| {
            s.direct_source().unwrap_or(Source::MyVariant)
        }),
        ("get", "protein") => word
            .parse::<ProteinSection>()
            .ok()
            .and_then(|s| s.source())
            .map(|s| vec![s]),
        ("get", "article") => many(word, ArticleSection::sources),
        ("get", "cell-line") => many(word, CellLineSection::sources),
        ("get", "disease") => many(word, DiseaseSection::sources),
        ("get", "pathway") => many(word, PathwaySection::sources),
        ("get", "phenotype") => many(word, PhenotypeSection::sources),
        ("get", "pgx") => one(word, PgxSection::source),
        ("get", "adverse-event") => one(word, |s: AdverseEventSection| s.source().source()),
        ("search", "adverse-event") => one(word, |_: VaersSection| Source::Vaers),
        ("get", "drug") => word.parse::<DrugSection>().ok().map(|s| {
            DrugSource::ALL
                .into_iter()
                .filter(|source| source.sections().contains(&s))
                .map(DrugSource::source)
                .collect()
        }),
        ("get", "trial") => supported(word, candidates, |source, s: TrialSection| {
            source
                .to_string()
                .parse::<TrialSource>()
                .is_ok_and(|source| source.sections().contains(&s))
        }),
        ("get", "diagnostic") => supported(word, candidates, |source, s: DiagnosticSection| {
            source
                .to_string()
                .parse::<DiagnosticGetSource>()
                .is_ok_and(|source| source.sections().contains(&s))
        }),
        _ => None,
    }
}
fn one<T: std::str::FromStr>(word: &str, source: impl FnOnce(T) -> Source) -> Option<Vec<Source>> {
    word.parse().ok().map(|section| vec![source(section)])
}
fn many<T: std::str::FromStr>(
    word: &str,
    sources: impl FnOnce(T) -> &'static [Source],
) -> Option<Vec<Source>> {
    word.parse().ok().map(|section| sources(section).to_vec())
}
fn supported<T: std::str::FromStr + Copy>(
    word: &str,
    candidates: &[Source],
    accepts: impl Fn(Source, T) -> bool,
) -> Option<Vec<Source>> {
    word.parse().ok().map(|section| {
        candidates
            .iter()
            .copied()
            .filter(|source| accepts(*source, section))
            .collect()
    })
}
