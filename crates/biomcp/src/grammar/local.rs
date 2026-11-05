use super::bundle_names;
use clap::{Arg, ArgAction, Command};
pub(super) fn serve_command() -> Command {
    Command::new("serve")
        .arg(Arg::new("tool-mode").long("tool-mode")
            .help("Advertise one CLI tool or generated typed command tools; default single.")
            .value_parser(clap::value_parser!(crate::config::ToolMode)))
        .after_help("Example: biomcp serve\nExample: biomcp help serve")
        .arg(
            super::boolean_argument(Arg::new("hosted")
                .long("hosted")
                .help("Refuse host file saves, health details and store or key changes over MCP.")
                ),
        )
        .arg(
            Arg::new("rate-share")
                .long("rate-share")
                .help("Use this share of keyed source rates in hosted mode (greater than 0, at most 1).")
                .value_parser(clap::value_parser!(biomcp_fetch::RateShare))
                .default_value("1")
                .requires("hosted"),
        )
        .arg(
            Arg::new("http")
                .long("http")
                .help("Listen on an HTTP address; omit its value for the loopback default.")
                .num_args(0..=1)
                .default_missing_value("127.0.0.1:8080"),
        )
        .arg(
            Arg::new("http_max_calls")
                .long("http_max_calls")
                .help("Limit tool calls in flight.")
                .value_parser(clap::value_parser!(std::num::NonZeroUsize))
                .requires("http"),
        )
        .arg(
            Arg::new("http_max_connections")
                .long("http_max_connections")
                .help("Limit open HTTP connections; must cover http_max_calls.")
                .value_parser(clap::value_parser!(std::num::NonZeroUsize))
                .requires("http"),
        )
        .arg(
            Arg::new("allow-host")
                .long("allow-host")
                .help("Allow an HTTP Host authority; required for non-loopback listeners.")
                .action(ArgAction::Append)
                .requires("http"),
        )
}

pub(super) fn help_command(root: &Command) -> Command {
    let mut help = Command::new("help");
    for command in root.get_subcommands() {
        let mut child = help_tree(command);
        if command.get_name() == "get" {
            child = child.subcommand(Command::new("clingen"));
        }
        if command.get_name() == "key" {
            child = child.arg(
                Arg::new("source")
                    .help("Credential source to describe.")
                    .value_parser(crate::keys::names()),
            );
        }
        if command.get_name() == "sync" {
            child = child.subcommands(bundle_names().into_iter().map(Command::new));
        }
        help = help.subcommand(child);
    }
    help.subcommand(
        Command::new("help").arg(
            Arg::new("command")
                .num_args(0..)
                .help("Command help path to describe."),
        ),
    )
    .subcommand(
        Command::new("network").arg(
            Arg::new("setting")
                .help("Network setting to describe.")
                .value_parser(["ca_bundle", "proxy"]),
        ),
    )
    .subcommand(Command::new("sources"))
    .subcommand(Command::new("config"))
    .subcommand(Command::new("exit").about("Explain process exit and error codes."))
    .subcommand(
        Command::new("relations").about("Map entities through sections and search filters."),
    )
}

fn help_tree(command: &Command) -> Command {
    Command::new(command.get_name().to_owned())
        .subcommands(command.get_subcommands().map(help_tree))
}

pub(super) fn help_source(
    matches: &clap::ArgMatches,
) -> Result<biomcp_fetch::SecretName, super::ParseInvalid> {
    let word = matches
        .subcommand()
        .filter(|(name, _)| *name == "key")
        .and_then(|(_, matches)| matches.get_one::<String>("source"));
    match word {
        Some(word) => biomcp_query::sources::keys()
            .into_iter()
            .find(|&name| biomcp_query::sources::key_name(name) == word)
            .ok_or_else(super::ParseInvalid::default),
        None => Ok(biomcp_fetch::SecretName::OncokbApiKey),
    }
}

pub(super) fn housekeeping_commands() -> [Command; 3] {
    [
        crate::skill::grammar(),
        Command::new("version").after_help("Example: biomcp version")
            .about("Print version; --verbose adds build, schema and path provenance. No source requests or key reads.")
            .arg(
                super::boolean_argument(Arg::new("verbose")
                    .long("verbose")
                    .help("Include commit, target, schema versions, config and cache paths.")
                    ),
            ),
        Command::new("mcp-config").after_help("Example: biomcp mcp-config")
            .about(
                "Print MCP configuration with this executable’s absolute path. JSON formats name their top-level \
                object and retain released client aliases. The codex client prints its shell registration command. \
                No source requests or key reads.",
            )
            .arg(
                Arg::new("client")
                    .help("Choose a JSON client or the codex shell registration command; default mcp-servers.")
                    .long("client")
                    .value_parser(clap::value_parser!(crate::housekeeping::Client))
                    .default_value("mcp-servers"),
            ),
    ]
}
pub(super) fn housekeeping(
    verb: &str,
    matches: &clap::ArgMatches,
) -> Option<crate::housekeeping::Command> {
    use crate::housekeeping::{Client, Command};
    match verb {
        "skill" => crate::skill::parse(matches).map(Command::Skill),
        "version" => Some(Command::Version {
            verbose: matches.get_flag("verbose"),
        }),
        "mcp-config" => matches
            .get_one::<Client>("client")
            .map(|&client| Command::McpConfig { client }),
        _ => None,
    }
}

pub(super) fn serve(matches: &clap::ArgMatches, mut globals: super::Globals) -> super::Parsed {
    globals.mcp_tool_mode = matches
        .get_one::<crate::config::ToolMode>("tool-mode")
        .copied();
    globals.hosted = matches.get_flag("hosted");
    globals.http_max_calls = matches
        .get_one::<std::num::NonZeroUsize>("http_max_calls")
        .copied();
    globals.http_max_connections = matches
        .get_one::<std::num::NonZeroUsize>("http_max_connections")
        .copied();
    globals.rate_share = matches
        .get_one::<biomcp_fetch::RateShare>("rate-share")
        .copied()
        .unwrap_or_default();
    super::Parsed {
        key: None,
        housekeeping: None,
        http: matches.get_one::<String>("http").map(|address| {
            (
                address.clone(),
                matches
                    .get_many::<String>("allow-host")
                    .into_iter()
                    .flatten()
                    .cloned()
                    .collect(),
            )
        }),
        action: None,
        globals,
        help: None,
    }
}
