//! Keep adapter diagnostics beside mapped associations.
pub(crate) fn association(
    row: biomcp_adapters::gwas::Association,
    problems: &mut Vec<biomcp_types::Problem>,
) -> biomcp_types::GwasAssociation {
    problems.extend(row.problems);
    row.value
}
