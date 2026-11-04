use super as runner;
use std::path::Path;

pub(crate) fn disease_native(path: &Path) -> Result<(), String> {
    runner::disease_ols::native(path)
}
pub(crate) fn mydisease_native(path: &Path) -> Result<(), String> {
    runner::disease_mydisease::native(path)
}
pub(crate) fn seer_native(path: &Path) -> Result<(), String> {
    runner::disease_seer::native(path)
}
pub(crate) fn trial_condition_native(path: &Path) -> Result<(), String> {
    runner::trial_condition::native(path)
}
pub(crate) fn monarch_native(path: &Path) -> Result<(), String> {
    runner::disease_monarch::native(path)
}
pub(crate) fn faers_search_examples(directory: &std::path::Path) -> Result<(), String> {
    runner::faers_search::record_examples(directory)
}
pub(crate) fn phenotype_native(path: &Path) -> Result<(), String> {
    runner::phenotype::native(path)
}
pub(crate) fn protein_native(path: &Path) -> Result<(), String> {
    runner::protein_commands::native(path)
}

pub(crate) fn other_source_values(path: &Path) -> Result<(), String> {
    runner::other_source_value_proof::run(path)
}
