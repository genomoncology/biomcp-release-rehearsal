//! Public command descriptions shared by parser help and generated artifacts.
use clap::Command;
pub(super) fn describe(mut command: Command) -> Command {
    let description = match command.get_name() {
        "biomcp" => "Retrieve source records for biomedical evidence questions.",
        "search" => "Search source records by entity and checked filters.",
        "get" => "Retrieve one entity and its source sections.",
        "article" => "Read article text, references, citations and files.",
        "trial" => "Read trial groups and public documents.",
        "variant" => "Read source variant assertions, populations and recurrence.",
        "protein" => "Read protein sequences, features and source relationships.",
        "pathway" => "Read pathway genes and source events.",
        "cell-line" => "Read source cell line assay counts.",
        "author" => "Read papers attributed to a source author record.",
        "sync" => "Synchronize a local source bundle.",
        "serve" => "Serve MCP over standard input or HTTP.",
        "key" => "Register and manage source credentials without showing values.",
        "list" => "List supported discovery topics.",
        "store" => "Manage local cached replies and snapshots.",
        "help" => "Describe commands and their arguments.",
        "health" => "Report source terms and bounded reachability.",
        _ => "",
    };
    if command.get_about().is_none() && !description.is_empty() {
        command = command.about(description);
    }
    let children: Vec<_> = command
        .get_subcommands()
        .map(|child| child.get_name().to_owned())
        .collect();
    for child in children {
        command = command.mut_subcommand(child, describe);
    }
    command
}

/// Source-specific replacement filters; error formatting consumes this declaration.
pub(super) const FILTER_EQUIVALENTS: &[(&str, &str, &str, &str, &str)] =
    &[("search", "diagnostic", "fda-cdx", "gene", "biomarker")];
