use super as runner;
use libtest_mimic::{Arguments, Trial};
use std::{path::PathBuf, process::ExitCode};

pub(crate) fn main(args: Vec<String>) -> ExitCode {
    match trials(args) {
        Ok((arguments, trials)) => {
            // libtest-mimic 0.8.2 exposes its own exact/skip/ignored selector.
            // Selected ignored tests remain selected; --list has no result counts.
            if let Some(filter) = &arguments.filter
                && trials.iter().all(|trial| arguments.is_filtered_out(trial))
            {
                use std::io::Write;
                let _ = writeln!(
                    std::io::stderr(),
                    "cases: filter {filter:?} selected no tests"
                );
                return ExitCode::FAILURE;
            }
            libtest_mimic::run(&arguments, trials).exit_code()
        }
        Err(error) => {
            use std::io::Write;
            let _ = writeln!(std::io::stderr(), "cases: {error}");
            ExitCode::FAILURE
        }
    }
}

fn trials(arguments: Vec<String>) -> Result<(Arguments, Vec<Trial>), String> {
    let mut args = arguments.into_iter();
    let mut cargo_args = vec![args.next().unwrap_or_else(|| "cases".into())];
    let mut directory = None;
    while let Some(argument) = args.next() {
        if argument == "--cases" {
            if directory.is_some() {
                return Err("--cases may appear only once".into());
            }
            directory = Some(PathBuf::from(
                args.next().ok_or("--cases requires a directory")?,
            ));
        } else {
            cargo_args.push(argument);
        }
    }
    let directory = directory.unwrap_or_else(|| runner::launch::fixtures().join("cases"));
    let mut trials = runner::discover(&directory)?;
    trials.extend(runner::protocol_trials());
    runner::validate_features(
        &trials,
        directory == runner::launch::fixtures().join("cases"),
    )?;
    tool_only! { trials.extend(super::tool_trials()?); }
    let arguments = Arguments::from_iter(cargo_args);
    tool_only! {
        if arguments.filter.is_none() && !arguments.list {
            return Err("tool case probes require a filter or --list".into());
        }
    }
    Ok((arguments, trials))
}
