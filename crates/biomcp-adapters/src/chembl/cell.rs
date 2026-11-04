//! Exact source-local cell metadata and assay counts.
use super::SPEC;
use crate::native_text::NativeText;
use biodata::{CellosaurusAccession, ChemblCellId};
use biomcp_fetch::Input;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, Reply, ReplyMetadata, RequestError};
use biomcp_types::{CellLineChemblRecord, CellLineRelease};
use serde::Deserialize;
const CELLS: biomcp_fetch::Endpoint = SPEC.get("/cell_line.json").fresh(Freshness::Days(1));
const ASSAYS: biomcp_fetch::Endpoint = SPEC.get("/assay.json").fresh(Freshness::Days(1));
const STATUS: biomcp_fetch::Endpoint = SPEC.get("/status.json").fresh(Freshness::Days(1));
/// Cell-specific native paging; drug endpoint validation remains independent.
#[derive(Debug, Deserialize)]
struct PageMeta {
    total_count: u64,
    offset: u32,
    limit: u32,
    next: Option<biodata::Text>,
    previous: Option<biodata::Text>,
}
impl PageMeta {
    fn check(&self, count: usize, limit: u32, offset: u32) -> Result<(), BadReply> {
        let expected = self
            .total_count
            .saturating_sub(u64::from(offset))
            .min(u64::from(limit));
        let more = u64::from(offset) + u64::from(limit) < self.total_count;
        if (1..=25).contains(&limit)
            && self.limit == limit
            && self.offset == offset
            && count as u64 == expected
            && self.next.is_some() == more
            && (offset != 0 || self.previous.is_none())
        {
            Ok(())
        } else {
            Err(BadReply)
        }
    }
}
#[derive(Debug, Deserialize)]
struct Cells {
    cell_lines: Vec<super::scope::Row<Cell, CellScope>>,
    page_meta: PageMeta,
}
#[derive(Debug, Deserialize)]
struct CellScope {
    cellosaurus_id: CellosaurusAccession,
}
#[derive(Debug, Deserialize)]
struct Cell {
    cell_chembl_id: ChemblCellId,
    cell_id: u64,
    cellosaurus_id: CellosaurusAccession,
    cell_name: Option<crate::raw::Field<NativeText>>,
    cell_description: Option<crate::raw::Field<NativeText>>,
    cell_source_organism: Option<crate::raw::Field<NativeText>>,
    cell_source_tax_id: Option<crate::raw::Field<u32>>,
    cell_source_tissue: Option<crate::raw::Field<NativeText>>,
    clo_id: Option<crate::raw::Field<biodata::Text>>,
    efo_id: Option<crate::raw::Field<biodata::Text>>,
}
impl Reply for Cells {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
impl Cell {
    fn identity_admitted(&self) -> bool {
        self.cell_id > 0
    }

    fn mapped(self, problems: &mut crate::mapped::Problems) -> CellLineChemblRecord {
        let r = self;
        problems.record = crate::mapped::record(&r.cell_chembl_id);
        let taxon = problems.field("cell_source_tax_id", r.cell_source_tax_id);
        let taxon = problems.optional(
            "cell_source_tax_id",
            taxon.map(|n| std::num::NonZeroU32::new(n).map(|n| n.get()).ok_or(())),
        );
        CellLineChemblRecord {
            id: r.cell_chembl_id,
            cell_id: biodata::Identifier {
                namespace: problems.text("cell_id.namespace", Some("ChEMBL")).into(),
                id: r.cell_id,
            },
            cellosaurus_id: r.cellosaurus_id,
            name: problems.field("cell_name", r.cell_name).map(Into::into),
            description: problems
                .field("cell_description", r.cell_description)
                .map(Into::into),
            organism: problems
                .field("cell_source_organism", r.cell_source_organism)
                .map(Into::into),
            taxon,
            tissue: problems
                .field("cell_source_tissue", r.cell_source_tissue)
                .map(Into::into),
            clo: problems.field("clo_id", r.clo_id),
            efo: problems.field("efo_id", r.efo_id),
        }
    }
}
/// Checked cell records, native total, retrieval metadata and native window size.
#[derive(Debug)]
pub struct CellPage {
    /// Accepted cells and their native window.
    pub window: crate::NativeWindow<CellLineChemblRecord>,
    /// Original acquisition metadata.
    pub metadata: ReplyMetadata,
    /// Independent field diagnostics.
    pub problems: Vec<biomcp_types::Problem>,
}
/// Read one native cell metadata page and check every exact accession before cache admission.
pub fn cells(
    leg: &Leg<'_>,
    id: &CellosaurusAccession,
    limit: u32,
    offset: u32,
) -> Result<CellPage, RequestError> {
    let text = id.to_string();
    let (reply, metadata) = leg
        .call(
            &CELLS,
            Input::Params(&[
                ("cellosaurus_id", Arg::Text(&text)),
                ("limit", Arg::Num(limit.into())),
                ("offset", Arg::Num(offset.into())),
            ]),
            biomcp_fetch::Read::json()
                .check(&|r: &Cells| {
                    r.page_meta.check(r.cell_lines.len(), limit, offset)?;
                    if r.cell_lines.iter().all(|r| &r.scope.cellosaurus_id == id) {
                        Ok(())
                    } else {
                        Err(BadReply)
                    }
                })
                .cache_if(&|r| {
                    let accepted = r
                        .cell_lines
                        .iter()
                        .filter(|row| row.value.as_ref().is_some_and(Cell::identity_admitted))
                        .count();
                    let refusals = if accepted < r.cell_lines.len() {
                        crate::Refusal::Refused
                    } else {
                        crate::Refusal::None
                    };
                    refusals.identities_admitted(accepted, r.cell_lines.len())
                }),
        )
        .map(|received| (received.value, received.meta))?;
    let received = reply.cell_lines.len() as u32;
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Chembl,
        record: crate::mapped::record(id),
        list: Vec::new(),
    };
    let mut values = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in reply.cell_lines {
        if let Some(row) = problems.field("cell_chembl_id", Some(row.value)) {
            if !row.identity_admitted() {
                refusals = crate::Refusal::Refused;
                problems.record = crate::mapped::record(&row.cell_chembl_id);
                problems.required::<()>("cell_id", None);
            } else {
                values.push(row.mapped(&mut problems));
            }
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    Ok(CellPage {
        window: crate::NativeWindow::new(
            values,
            offset,
            limit,
            Some(reply.page_meta.total_count),
            received,
            refusals,
        ),
        metadata,
        problems: problems.list,
    })
}
#[derive(Debug, Deserialize)]
struct Assays {
    assays: Vec<Assay>,
    page_meta: PageMeta,
}
#[derive(Debug, Deserialize)]
struct Assay {
    cell_chembl_id: ChemblCellId,
}
impl Reply for Assays {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
/// Read only the native assay total; validate every returned cell identity first.
pub fn assay_count(leg: &Leg<'_>, id: &ChemblCellId) -> Result<(u64, ReplyMetadata), RequestError> {
    let text = id.to_string();
    let (reply, metadata) = leg
        .call(
            &ASSAYS,
            Input::Params(&[
                ("cell_chembl_id", Arg::Text(&text)),
                ("limit", Arg::Num(1)),
                ("offset", Arg::Num(0)),
            ]),
            biomcp_fetch::Read::json().check(&|r: &Assays| {
                r.page_meta.check(r.assays.len(), 1, 0)?;
                if (r.page_meta.total_count == 0) == r.assays.is_empty()
                    && r.assays.iter().all(|r| &r.cell_chembl_id == id)
                {
                    Ok(())
                } else {
                    Err(BadReply)
                }
            }),
        )
        .map(|received| (received.value, received.meta))?;
    Ok((reply.page_meta.total_count, metadata))
}
#[derive(Debug, Deserialize)]
struct Release {
    chembl_db_version: Option<crate::raw::Field<biodata::Text>>,
    chembl_release_date: Option<crate::raw::Field<biodata::PartialDate>>,
}
impl Reply for Release {
    fn check(&self) -> Result<(), BadReply> {
        Ok(())
    }
}
/// Read publisher release independently from data retrieval time.
pub fn release(leg: &Leg<'_>) -> Result<crate::Mapped<CellLineRelease>, RequestError> {
    let r: Release = leg
        .call(&STATUS, Input::Params(&[]), biomcp_fetch::Read::json())?
        .value;
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Chembl,
        record: None,
        list: Vec::new(),
    };
    let value = CellLineRelease {
        version: problems.required_field("chembl_db_version", r.chembl_db_version),
        updated: problems.required_field("chembl_release_date", r.chembl_release_date),
    };
    Ok(crate::Mapped {
        refusals: crate::Refusal::None,
        value,
        problems: problems.list,
    })
}
