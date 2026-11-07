//! The BioMCP command entry point.
mod command;
mod config;
mod exit;
mod grammar;
mod help;
mod housekeeping;
mod keys;
mod mcp;
mod output;
mod paths;
mod retired;
mod save;
mod schema;
mod skill;
mod spell;
mod view;

use std::process::ExitCode;

fn main() -> ExitCode {
    let root = match grammar::root() {
        Ok(root) => std::sync::Arc::new(root),
        Err(_) => return exit::failure(std::io::Error::other("command grammar unavailable")),
    };
    let args = grammar::entity_first(&root, config::arguments());
    let parsed = grammar::parse(&root, &args);
    if let Ok(parsed) = &parsed
        && let Some(command) = &parsed.housekeeping
    {
        return match command.render(&parsed.globals) {
            Ok(bytes) => exit::finish(output::bytes(&bytes), exit::Status::Success),
            Err(error) => exit::failure(error),
        };
    }
    let settings = std::sync::Arc::new(if grammar::plain_housekeeping(&root, &args) {
        Ok(None)
    } else {
        let (_, path) = grammar::presentation(&root, &args).unwrap_or_default();
        config::file::read_file(path.as_ref())
    });
    if let Ok(parsed) = parsed
        && parsed.action.is_none()
        && parsed.key.is_none()
    {
        let globals = parsed.globals;
        if let Some((address, hosts)) = parsed.http {
            return match mcp::http::serve(&address, hosts, globals, root.clone(), settings.clone())
            {
                Ok(()) => ExitCode::SUCCESS,
                Err(mcp::http::StartError::Configuration(error)) => {
                    match command::local::startup(
                        error,
                        command::Encoding::cli(&root, &settings, &args),
                    ) {
                        Ok(outcome) => exit::finish(output::command(&outcome), outcome.status),
                        Err(error) => exit::failure(error),
                    }
                }
                Err(mcp::http::StartError::AllowHost) => exit::finish(
                    command::Encoding::Json
                        .encode(
                            biomcp_types::ErrorEnvelope::from(biomcp_types::UsageError::argument(
                                "--allow-host".into(),
                            )),
                            exit::Status::Usage,
                        )
                        .and_then(|outcome| output::command(&outcome)),
                    exit::Status::Usage,
                ),
                Err(mcp::http::StartError::Usage) => ExitCode::from(exit::Status::Usage.code()),
                Err(mcp::http::StartError::Local) => ExitCode::from(exit::Status::Local.code()),
            };
        }
        let fetch = config::fetch(&globals, &settings);
        if let Err(error) = &fetch
            && error.is_pacing()
        {
            return match command::local::startup(
                error.clone(),
                command::Encoding::cli(&root, &settings, &args),
            ) {
                Ok(outcome) => exit::finish(output::command(&outcome), outcome.status),
                Err(error) => exit::failure(error),
            };
        }
        return exit::finish(
            mcp::serve(globals, fetch, root.clone(), settings.clone()),
            exit::Status::Success,
        );
    }
    match command::dispatch(&root, &settings, &args) {
        Ok(outcome) => exit::finish(output::command(&outcome), outcome.status),
        Err(error) => exit::failure(error),
    }
}
