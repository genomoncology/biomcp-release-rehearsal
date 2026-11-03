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
