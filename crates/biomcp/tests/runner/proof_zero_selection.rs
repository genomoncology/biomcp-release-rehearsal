//! Focused proof runs refuse a selection that matches nothing.
use super::super::launch::Environment;
use std::env;

pub(super) fn check() -> Result<(), String> {
    let executable = env::current_exe().map_err(|error| error.to_string())?;
    for options in [
        vec!["--proofs", "no-such-runner-selection-7e41", "--exact"],
        vec!["--proofs", "schema-discovery/selected.yaml", "--exact"],
        vec!["--proofs", "--skip", ""],
    ] {
        let output = Environment::new()
            .map_err(|error| error.to_string())?
            .command(&executable, &options.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>())
            .output()
            .map_err(|error| error.to_string())?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() || !stderr.contains("--cases") || stderr.contains("Some(") {
            return Err(format!("empty proof selection must fail and suggest --cases: {stderr}; {}",
                String::from_utf8_lossy(&output.stdout)));
        }
    }
    let output = Environment::new()
        .map_err(|error| error.to_string())?
        .command(&executable, &["--proofs".into(), "no-such-runner-selection-7e41".into(), "--list".into()])
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("empty filtered proof listing must remain successful".into());
    }
    for (args, empty) in [
        (vec!["cases", "--skip", "selected.yaml"], true),
        (vec!["cases", "--ignored"], true),
        (vec!["cases", "--skip", "selected.yaml", "--list"], false),
    ] {
        let arguments = libtest_mimic::Arguments::from_iter(args.iter().copied());
        let trials = [libtest_mimic::Trial::test("selected.yaml", || Ok(()))];
        if super::super::case_main::selects_nothing(&arguments, &trials) != empty {
            return Err(format!("no-filter selection {args:?} must report empty={empty}"));
        }
    }
    Ok(())
}
