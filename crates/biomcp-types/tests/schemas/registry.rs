//! Register shared value and research query schema contracts.
use super::*;

pub(super) fn common_registry() -> Vec<Contract> {
    contracts! {
        Accession => ("accession", "biodata.accession/3"),
        DrugRecallNumber => ("drug-recall-number", "biodata.drug-recall-number/1"),
        DeviceRecallNumber => ("device-recall-number", "biodata.device-recall-number/1"),
        Chromosome => ("chromosome", "biodata.chromosome/1"),
        Base => ("base", "biodata.base/1"),
        Sequence => ("sequence", "biodata.sequence/1"),
        SequenceLocation => ("sequence-location", "biodata.sequence-location/3"),
        Allele => ("allele", "biodata.allele/3"),
        CodingChange => ("coding-change", "biodata.coding-change/1"),
        HgvsText => ("hgvs-text", "biodata.hgvs-text/4"),
        Expression => ("expression", "biodata.expression/3"),
        ExpressionSyntax => ("expression-syntax", "biodata.expression-syntax/1"),
        Representation => ("representation", "biodata.representation/5"),
        VariantId => ("variant-id", "biodata.variant-id/3"),
        Variant => ("variant", "biodata.variant/7"),
        RsId => ("rs-id", "biodata.rs-id/2"),
        CaId => ("ca-id", "biodata.ca-id/2"),
        PaId => ("pa-id", "biodata.pa-id/2"),
        VcvId => ("vcv-id", "biodata.vcv-id/2"),
        RcvId => ("rcv-id", "biodata.rcv-id/2"),
        ScvId => ("scv-id", "biodata.scv-id/2"),
        ClinVarVariationId => ("clinvar-variation-id", "biodata.clinvar-variation-id/2"),
        ClinVarAlleleId => ("clinvar-allele-id", "biodata.clinvar-allele-id/2"),
        CosmicId => ("cosmic-id", "biodata.cosmic-id/1"),
        MondoId => ("mondo-id", "biodata.mondo-id/1"),
        NcitId => ("ncit-id", "biodata.ncit-id/1"),
        VrsId => ("vrs-id", "biodata.vrs-id/1"),
        Contig => ("contig", "biodata.contig/1"),

    }
}

pub(super) fn research_registry() -> Vec<Contract> {
    contracts! {
        ResearchSearch<ResearchEnvelope> => ("research-search", ResearchSearch::<ResearchEnvelope>::SCHEMA),
        ResearchFetch<ResearchEnvelope> => ("research-fetch", ResearchFetch::<ResearchEnvelope>::SCHEMA),
    }
}
