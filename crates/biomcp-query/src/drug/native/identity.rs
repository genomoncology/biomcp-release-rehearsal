use biodata::Error;
use biomcp_adapters::chembl;
use biomcp_types::{DrugCard, DrugHeader};
pub(super) fn chembl_header(
    row: &chembl::Molecule,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Result<DrugHeader, Error> {
    let mapped = chembl::molecule_header(row)?;
    problems.extend(mapped.problems);
    Ok(mapped.value)
}
pub(super) fn chembl_card(
    row: chembl::Molecule,
    fetched: Option<i64>,
    dropped: &mut bool,
    problems: &mut Vec<biomcp_types::Problem>,
) -> Result<DrugCard, Error> {
    let mapped = chembl::molecule_card(row, fetched)?;
    *dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
    problems.extend(mapped.problems);
    Ok(mapped.value)
}
