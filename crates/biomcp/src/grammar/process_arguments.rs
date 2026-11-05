//! Process options share one declaration with installed replay discovery.
use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

/// Shared literal semantics for actual booleans; fixed-value enum switches opt out.
pub(crate) fn boolean_argument(argument: Arg) -> Arg {
    argument
        .action(ArgAction::SetTrue)
        .num_args(0..=1)
        .require_equals(true)
        .default_missing_value("true")
}

pub(crate) fn boolean_value(argument: &Arg, value: Option<&std::ffi::OsString>) -> bool {
    matches!(argument.get_action(), ArgAction::SetTrue)
        && argument.is_require_equals_set()
        && value.is_some_and(|value| matches!(value.to_str(), Some("true" | "false")))
}

pub(crate) fn command() -> Command {
    let mut root = Command::new("biomcp")
        .disable_help_flag(true)
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg_required_else_help(true)
        .subcommand_required(true);
    root = root.arg(
        Arg::new("format")
            .long("format")
            .help("Choose JSON or Markdown output; configuration can change the default.")
            .default_value("json")
            .global(true)
            .value_parser(["json", "md"]),
    );
    root = root.arg(
        Arg::new("disable-sources")
            .long("disable-sources")
            .help("Exclude source names or source terms groups. Example: --disable-sources civic.")
            .global(true)
            .num_args(1)
            .value_delimiter(',')
            .action(ArgAction::Append),
    );
    root = root.arg(
        Arg::new("source_pacing")
            .long("source_pacing")
            .help(
                "Override source pacing with a JSON map of rate, keyed_rate and in_flight values. Example: --source_pacing '{}'.",
            )
            .global(true),
    );
    for name in ["config", "replay", "record", "case-cache", "observe"] {
        root = root.arg(
            Arg::new(name)
                .long(name)
                .hide(name != "config")
                .help("Read configuration from this file. Example: --config config.json.")
                .global(true)
                .value_parser(clap::value_parser!(PathBuf)),
        );
    }
    for name in ["offline", "fresh"] {
        root = root.arg(boolean_argument(
            Arg::new(name)
                .long(name)
                .help(if name == "offline" {
                    "Read local data without source requests."
                } else {
                    "Request fresh source data instead of cached replies."
                })
                .global(true)
                .overrides_with(name),
        ));
    }
    root
}
