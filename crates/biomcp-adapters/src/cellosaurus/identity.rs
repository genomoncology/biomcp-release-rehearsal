//! Request-bound identity checks and reviewed native database names.
use super::raw;
use biodata::{CellLineSourceId, CellosaurusAccession};
use biomcp_fetch::BadReply;
pub(super) fn check_card(
    reply: &raw::Response,
    accession: &CellosaurusAccession,
    source: Option<&CellLineSourceId>,
) -> Result<(), BadReply> {
    match reply {
        raw::Response::Missing { code: 404, message }
            if message == &format!("Item not found, ac: {accession}") =>
        {
            Ok(())
        }
        raw::Response::Records { data } if data.rows.len() == 1 => {
            let row = data.rows.first().ok_or(BadReply)?;
            if row.accession()? != accession {
                return Err(BadReply);
            }
            if let Some(id) = source {
                let database = native_database(id).ok_or(BadReply)?;
                if !row
                    .xref_list
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .any(|r| {
                        r.database.as_str() == database && r.accession.as_str() == id.accession()
                    })
                {
                    return Err(BadReply);
                }
            }
            Ok(())
        }
        _ => Err(BadReply),
    }
}
/// Public supported source IDs map only to reviewed native database names.
pub fn native_database(id: &CellLineSourceId) -> Option<&'static str> {
    match id.database() {
        "DepMap" => Some("DepMap"),
        "CellModelPassports" | "Cell_Model_Passport" => Some("Cell_Model_Passport"),
        "ChEMBL" | "ChEMBL-Cells" => Some("ChEMBL-Cells"),
        "PharmacoDB" => Some("PharmacoDB"),
        _ => None,
    }
}
