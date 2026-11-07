//! Explicit measurement and generation tools.
#![expect(
    clippy::disallowed_methods,
    reason = "The test entry point reads arguments and locates its own executable"
)]
macro_rules! tool_only {
    ($($tokens:tt)*) => { $($tokens)* };
}
mod runner;

use std::{env, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    if let Some(code) = native_proof() {
        return code;
    }
    if env::args().nth(1).as_deref() == Some("--runner-environment-probe") {
        return runner::proof::environment_probe();
    }
    match env::args().nth(1).as_deref() {
        Some("--proof-metadata") => match runner::proof_requirements::metadata() {
            Ok(metadata) => {
                use std::io::Write;
                let _ = writeln!(std::io::stdout(), "{metadata}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                use std::io::Write;
                let _ = writeln!(std::io::stderr(), "proof metadata: {error}");
                ExitCode::FAILURE
            }
        },
        Some("--proofs") => {
            let arguments = std::iter::once("tools".to_owned()).chain(env::args().skip(2));
            match runner::tool_trials() {
                Ok(trials) => {
                    let arguments = libtest_mimic::Arguments::from_iter(arguments);
                    if !arguments.list
                        && trials.iter().all(|trial| arguments.is_filtered_out(trial))
                    {
                        use std::io::Write;
                        let _ = writeln!(
                            std::io::stderr(),
                            "tools: --proofs {} selected no tests; use --proofs --list for custom trials or --cases DIRECTORY for YAML cases",
                            runner::case_main::selector(&arguments)
                        );
                        return ExitCode::FAILURE;
                    }
                    libtest_mimic::run(&arguments, trials).exit_code()
                }
                Err(error) => {
                    use std::io::Write;
                    let _ = writeln!(std::io::stderr(), "tools: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Some("--cases") => runner::case_main::main(env::args().collect()),
        Some("--discover") => runner::case_main::main(vec!["tools".into(), "--list".into()]),
        _ => ExitCode::FAILURE,
    }
}

fn native_proof() -> Option<ExitCode> {
    let measure: Option<fn() -> Result<(), String>> = match env::args().nth(1).as_deref() {
        Some("--cell-line-section-release-measure") => {
            Some(runner::cell_line::measure_section_release)
        }
        Some("--cell-line-help-measure") => Some(runner::cell_line::measure_help),
        Some("--cell-line-sections-measure") => Some(runner::cell_line::measure_sections),
        Some("--cell-line-measure") => Some(runner::cell_line::measure),
        Some("--cell-line-measure-remaining") => Some(runner::cell_line::measure_remaining),
        Some("--cell-line-canonical-measure") => Some(runner::cell_line::measure_canonical),
        _ => None,
    };
    if let Some(measure) = measure {
        return Some(match measure() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                use std::io::Write;
                let _ = writeln!(std::io::stderr(), "{error}");
                ExitCode::FAILURE
            }
        });
    }
    let mode = env::args().nth(1)?;
    let run = match mode.as_str() {
        "--cell-line-xrefs" => runner::cell_line::generate_xrefs,
        "--cell-line-examples" => runner::cell_line::generate_examples,
        "--faers-search-synthetic-examples" => runner::tools_api::faers_search_examples,
        "--other-source-value-proof" => runner::tools_api::other_source_values,
        "--seer-native-proof" => runner::tools_api::seer_native,
        "--trial-condition-native-proof" => runner::tools_api::trial_condition_native,
        "--monarch-native-proof" => runner::tools_api::monarch_native,
        "--phenotype-native-proof" => runner::tools_api::phenotype_native,
        "--protein-native-proof" => runner::tools_api::protein_native,

        "--mydisease-native-proof" => runner::tools_api::mydisease_native,
        "--disease-native-proof" => runner::tools_api::disease_native,
        _ => return None,
    };
    Some(
        match env::args()
            .nth(2)
            .map(PathBuf::from)
            .ok_or("missing evidence directory".to_owned())
            .and_then(|p| run(&p))
        {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                use std::io::Write;
                let _ = writeln!(std::io::stderr(), "{error}");
                ExitCode::FAILURE
            }
        },
    )
}
