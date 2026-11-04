use super::{GwasAssociation, GwasGet, GwasLocus, Outcome, Request, StepKind, action};
use biomcp_types::Source;
pub(super) fn association(row: &GwasAssociation, out: &mut Outcome) {
    if let Some(study) = &row.study
        && let Some(id) =
            crate::next::checked(study.as_str().parse(), Source::Gwas, &mut out.problems)
    {
        out.next.push(crate::next::Step::new(
            StepKind::Pivot,
            &[biomcp_types::Source::Gwas],
            action(GwasGet::study(id), false),
        ));
    }
    for allele in &row.alleles {
        if let Some(q) = crate::next::checked(
            allele
                .rs_id
                .as_str()
                .parse()
                .map_err(|_| biomcp_types::QueryInvalid::new(&["input"]))
                .and_then(|input| {
                    biomcp_types::VariantGet::new(input, biomcp_types::VariantResolver::Car, None)
                }),
            Source::Gwas,
            &mut out.problems,
        ) {
            out.next.push(crate::next::Step::run(
                StepKind::Pivot,
                &[biomcp_types::Source::MyVariant],
                Request::GetVariant(q),
                false,
            ));
        }
    }
    if let Some(publication) = &row.publication
        && let Some(q) = crate::next::checked(
            publication
                .as_str()
                .parse()
                .map_err(|_| biomcp_types::QueryInvalid::new(&["input"]))
                .and_then(|input| biomcp_types::ArticleGet::new(vec![input].into())),
            Source::Gwas,
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::run(
            StepKind::Pivot,
            &[biomcp_types::Source::PubMed],
            Request::GetArticle(q),
            false,
        ));
    }
}
pub(super) fn loci(rows: &[GwasLocus], out: &mut Outcome) {
    let mut seen = std::collections::BTreeSet::new();
    for gene in rows.iter().flat_map(|r| &r.author_reported_genes) {
        for id in gene.ensembl_ids.iter().chain(&gene.entrez_ids) {
            let input = if id.as_str().bytes().all(|b| b.is_ascii_digit()) {
                crate::next::checked(
                    id.as_str().parse::<biodata::NcbiGeneId>(),
                    Source::Gwas,
                    &mut out.problems,
                )
                .map(|id| id.to_string())
            } else {
                crate::next::checked(
                    id.as_str().parse::<biodata::EnsemblGeneId>(),
                    Source::Gwas,
                    &mut out.problems,
                )
                .map(|id| id.to_string())
            };
            let Some(input) = input else { continue };
            if seen.insert(input.clone())
                && let Some(q) = crate::next::checked(
                    input
                        .parse()
                        .map_err(|_| biomcp_types::QueryInvalid::new(&["input"]))
                        .and_then(|input| biomcp_types::GeneGet::new(vec![input].into())),
                    Source::Gwas,
                    &mut out.problems,
                )
            {
                out.next.push(crate::next::Step::run(
                    StepKind::Pivot,
                    &[biomcp_types::Source::Hgnc],
                    Request::GetGene(q),
                    false,
                ));
            }
        }
    }
}
