use super::{
    Withdrawn,
    fields::{self, Fields},
    raw,
};
use crate::Mapped;
use biodata::{Gene, GeneStatus, GeneSymbol, HgncId, Text};

pub(super) fn genes(rows: Vec<raw::Gene>) -> Mapped<Vec<Gene>> {
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    let value = rows
        .into_iter()
        .filter_map(|row| {
            let mut fields = Fields {
                id: raw::cell(row.hgnc_id.as_deref()).parse().ok(),
                problems: Vec::new(),
            };
            let gene = match gene(row, &mut fields) {
                Some(gene) => Some(gene),
                None => {
                    refusals = crate::Refusal::Refused;
                    None
                }
            };
            problems.extend(fields.problems);
            gene
        })
        .collect();
    Mapped {
        refusals,
        value,
        problems,
    }
}
fn gene(row: raw::Gene, f: &mut Fields) -> Option<Gene> {
    let id = f.required("hgnc_id", f.id);
    let symbol = f.required(
        "symbol",
        GeneSymbol::new(raw::cell(row.symbol.as_deref())).ok(),
    );
    let name = if raw::cell(row.name.as_deref()).is_empty() {
        None
    } else {
        f.required(
            "name",
            Text::from_source(raw::cell(row.name.as_deref())).ok(),
        )
    };
    let locus_group = row
        .locus_group
        .as_deref()
        .filter(|v| !v.is_empty())
        .and_then(|group| f.required("locus_group", fields::group(group)));
    let status = f.required(
        "status",
        (raw::cell(row.status.as_deref()) == "Approved").then_some(GeneStatus::Approved),
    );
    // All required fields are checked before dropping a record.
    let (hgnc_id, symbol, status) = (id?, symbol?, status?);
    let locus_type = if raw::cell(row.locus_type.as_deref()).is_empty() {
        None
    } else {
        f.required(
            "locus_type",
            fields::locus(raw::cell(row.locus_type.as_deref())),
        )
    };
    let mane_select = if raw::cell(row.mane_select.as_deref()).is_empty() {
        None
    } else {
        f.required(
            "mane_select",
            fields::mane(raw::cell(row.mane_select.as_deref())),
        )
    };
    Some(Gene {
        hgnc_id,
        symbol,
        name,
        locus_group,
        status,
        locus_type,
        mane: mane_select.into_iter().collect(),
        location: (!raw::cell(row.location.as_deref()).is_empty())
            .then(|| raw::cell(row.location.as_deref()).into()),
        entrez_id: f.optional("entrez_id", raw::cell(row.entrez_id.as_deref())),
        ensembl_id: f.optional("ensembl_id", raw::cell(row.ensembl_gene_id.as_deref())),
        previous_symbols: f.list("previous_symbols", raw::cell(row.prev_symbol.as_deref())),
        aliases: f.list("aliases", raw::cell(row.alias_symbol.as_deref())),
        uniprot: f.list("uniprot", raw::cell(row.uniprot_ids.as_deref())),
        omim: f.list("omim", raw::cell(row.omim_id.as_deref())),
        spans: Box::new([]),
        summary: None,
    })
}
pub(super) fn withdrawn(rows: Vec<raw::Withdrawn>) -> Mapped<Vec<Withdrawn>> {
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    let value = rows
        .into_iter()
        .filter_map(|row| {
            let mut f = Fields {
                id: raw::cell(row.hgnc_id.as_deref()).parse().ok(),
                problems: Vec::new(),
            };
            let id = f.required("hgnc_id", f.id);
            let symbol = f.required(
                "symbol",
                GeneSymbol::new(raw::cell(row.symbol.as_deref())).ok(),
            );
            let hgnc_status = f.required(
                "hgnc_status",
                match raw::cell(row.status.as_deref()) {
                    "Entry Withdrawn" => Some(biodata::HgncStatus::EntryWithdrawn),
                    "Merged/Split" => Some(biodata::HgncStatus::MergedSplit),
                    _ => None,
                },
            );
            let merged_into = if raw::cell(row.merged.as_deref()).is_empty() {
                Vec::new()
            } else {
                raw::cell(row.merged.as_deref())
                    .split(", ")
                    .filter_map(|part| {
                        let id = part
                            .split('|')
                            .next()
                            .and_then(|id| id.parse::<HgncId>().ok());
                        f.required("merged_into", id)
                    })
                    .collect()
            };
            problems.extend(f.problems);
            let (Some(hgnc_id), Some(symbol), Some(hgnc_status)) = (id, symbol, hgnc_status) else {
                refusals = crate::Refusal::Refused;
                return None;
            };
            Some(Withdrawn {
                hgnc_id,
                symbol,
                hgnc_status,
                merged_into: merged_into.into(),
            })
        })
        .collect();
    Mapped {
        refusals,
        value,
        problems,
    }
}
