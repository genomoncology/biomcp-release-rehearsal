//! Embedded, case-backed agent guidance with explicit local installation.
mod install;
mod status;
use clap::{Arg, Command};
use std::{io, path::PathBuf};
pub(super) const ASSETS: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/skills.rs"));
#[derive(Debug)]
pub(crate) enum Skill {
    Overview,
    List,
    Show(String),
    Install { directory: PathBuf, force: bool },
    Status(PathBuf),
}
pub(crate) fn grammar() -> Command {
    let directory = || {
        Arg::new("directory")
            .help("Directory to inspect or install BioMCP guidance.")
            .required(true)
            .value_parser(clap::value_parser!(PathBuf))
    };
    Command::new("skill").after_help("Example: biomcp skill list")
        .about("Read embedded guidance and worked examples. No source requests. \
            Overview and catalog are bounded by the embedded payload; install and status return one summary. \
            Install writes only inside DIR and refuses symbolic links and parent traversal.")
        .subcommand(Command::new("list").after_help("Example: biomcp skill list").about("List embedded worked examples."))
        .subcommand(Command::new("render").after_help("Example: biomcp skill render").about("Print the canonical agent guide."))
        .subcommand(Command::new("status").after_help("Example: biomcp skill status biomcp-guidance")
            .about("Report missing, unmanaged, locally_modified, stale or current guidance.")
            .arg(directory()))
        .subcommand(Command::new("install")
            .after_help("Example: biomcp skill install biomcp-guidance")
            .about("Install into DIR/skills/biomcp; a skills or biomcp directory is accepted directly. \
            Existing files remain unchanged without --force.")
            .arg(directory())
            .arg(crate::grammar::boolean_argument(Arg::new("force").long("force").help("Repair managed files; retain unrelated files."))))
        .allow_external_subcommands(true)
}
pub(crate) fn parse(matches: &clap::ArgMatches) -> Option<Skill> {
    Some(match matches.subcommand() {
        None | Some(("render", _)) => Skill::Overview,
        Some(("list", _)) => Skill::List,
        Some(("install", args)) => Skill::Install {
            directory: args.get_one::<PathBuf>("directory")?.clone(),
            force: args.get_flag("force"),
        },
        Some(("status", args)) => Skill::Status(args.get_one::<PathBuf>("directory")?.clone()),
        Some((name, args)) => {
            if args
                .get_many::<std::ffi::OsString>("")
                .is_some_and(|mut values| values.next().is_some())
            {
                return None;
            }
            Skill::Show(name.into())
        }
    })
}
impl Skill {
    pub(crate) fn render(&self) -> io::Result<String> {
        match self {
            Self::Overview => Ok(body(asset("SKILL.md")?).into()),
            Self::List => {
                let mut text = String::from("# BioMCP Worked Examples\n\n");
                for (path, contents) in examples() {
                    let name = path
                        .trim_start_matches("use-cases/")
                        .trim_end_matches(".md");
                    let title = contents
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .trim_start_matches("# ");
                    text.push_str(&format!("{name}: {title}\n"));
                }
                Ok(text)
            }
            Self::Show(name) => show(name),
            Self::Install { directory, force } => install::run(directory, *force),
            Self::Status(directory) => status::render(directory),
        }
    }
}
fn asset(name: &str) -> io::Result<&'static str> {
    ASSETS
        .iter()
        .find(|(path, _)| *path == name)
        .map(|(_, text)| *text)
        .ok_or_else(|| io::Error::other("unknown skill; use biomcp skill list"))
}
fn body(text: &str) -> &str {
    text.strip_prefix("---\n")
        .and_then(|text| {
            text.split_once("\n---\n")
                .map(|(_, body)| body.trim_start())
        })
        .unwrap_or(text)
}
fn examples() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    ASSETS.iter().filter(|(path, _)| {
        path.starts_with("use-cases/") && path.ends_with(".md") && !path.contains("/_")
    })
}
fn show(name: &str) -> io::Result<String> {
    let name = name.to_ascii_lowercase();
    let numeric = name
        .parse::<u32>()
        .ok()
        .map(|number| format!("{number:02}"));
    examples()
        .find(|(path, _)| {
            let stem = path
                .trim_start_matches("use-cases/")
                .trim_end_matches(".md");
            let (number, slug) = stem.split_once('-').unwrap_or((stem, stem));
            name == stem || name == slug || numeric.as_deref() == Some(number)
        })
        .map(|(_, text)| (*text).to_owned())
        .ok_or_else(|| io::Error::other("unknown skill; use biomcp skill list"))
}
