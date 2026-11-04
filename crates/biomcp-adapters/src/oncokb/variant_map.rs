//! Checked request identity survives malformed optional annotation content.
#[path = "variant_labels.rs"]
mod labels;
use super::{variant::Annotation, variant_groups as groups};
use crate::{Mapped, mapped::Problems};
use biodata::{Text, VariantRef};
use biomcp_types::{PageInfo, Source, VariantOncokb};

/// Map native values without attaching grouped implications to the queried allele.
pub fn records(mut row: Annotation, tumor_type: Text) -> Mapped<VariantOncokb> {
    let mut cx = Problems {
        source: Source::Oncokb,
        record: Some(row.query.hgvs.clone()),
        list: Vec::new(),
    };
    let hugo_symbol = cx.field("query.hugoSymbol", row.query.hugo_symbol.take());
    let alteration = cx.field("query.alteration", row.query.alteration.take());
    let gene = hugo_symbol
        .clone()
        .and_then(|value| cx.optional("query.hugoSymbol", Some(value.try_into())));
    let subject = gene
        .zip(alteration.clone())
        .map(|(gene, alteration)| VariantRef::protein(gene, alteration));
    let release = crate::mapped::optional_text(row.data_version.take(), "dataVersion", &mut cx);
    let mut records = labels::records(
        row.oncogenic.take(),
        row.mutation_effect.take(),
        subject.as_ref(),
        release.as_ref(),
        &mut cx,
    );
    super::narrative::annotation(&mut row, &mut cx, &mut records);
    records.extend(groups::treatments(row.treatments.take(), &mut cx));
    records.extend(groups::implications(
        row.diagnostic_implications.take(),
        "diagnosticImplications",
        &mut cx,
    ));
    records.extend(groups::implications(
        row.prognostic_implications.take(),
        "prognosticImplications",
        &mut cx,
    ));
    let total = records.len() as u64;
    Mapped {
        refusals: crate::Refusal::None,
        value: VariantOncokb {
            hgvsg: row.query.hgvs,
            build: row.query.reference_genome,
            tumor_type,
            hugo_symbol,
            alteration,
            data_version: release,
            last_update: crate::mapped::optional_text(row.last_update, "lastUpdate", &mut cx),
            variant_exist: cx.field("variantExist", row.variant_exist),
            gene_exist: cx.field("geneExist", row.gene_exist),
            vus: cx.field("vus", row.vus),
            hotspot: cx.field("hotspot", row.hotspot),
            allele_exist: cx.field("alleleExist", row.allele_exist),
            records: records.into(),
            page: PageInfo {
                offset: 0,
                returned: total as u32,
                total,
                pool: None,
                truncated: None,
            },
            next: None,
        },
        problems: cx.list,
    }
}
