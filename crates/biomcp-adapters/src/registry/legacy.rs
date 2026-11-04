//! Exact retired restricted group, retained only for existing CLI/config exclusions.
use biomcp_types::Source;
pub(super) fn sources() -> impl Iterator<Item = Source> {
    [
        Source::AlphaGenome,
        Source::ClinicalTrialsGov,
        Source::Nci,
        Source::AlleleRegistry,
        Source::Cbioportal,
        Source::ClinVar,
        Source::Erepo,
        Source::EuropePmc,
        Source::Gwas,
        Source::CancerHotspots,
        Source::LitSense,
        Source::Mutalyzer,
        Source::MyGene,
        Source::Oncokb,
        Source::PmcOa,
        Source::PubMed,
        Source::PubTator,
        Source::Vaers,
        Source::VariantValidator,
        Source::WhoPq,
    ]
    .into_iter()
}
pub(crate) fn component(source: Source, name: &str) -> bool {
    matches!(
        (source, name),
        (Source::MyVariant, "cosmic" | "clinvar" | "snpeff")
            | (Source::MyDisease, "omim")
            | (Source::Faers, "meddra")
            | (Source::Maude, "gmdn")
    )
}
