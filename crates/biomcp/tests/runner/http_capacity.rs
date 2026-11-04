//! Operator capacity settings reach the HTTP startup boundary.
use super::{http::Server, launch::Environment};
use serde_json::json;
pub(super) fn check() -> Result<(), String> {
    for (config, flags) in [
        (json!({"http_max_calls":2,"http_max_connections":4}), vec![]),
        (
            json!({"http_max_calls":6,"http_max_connections":1}),
            vec![
                "--http_max_calls".into(),
                "2".into(),
                "--http_max_connections".into(),
                "4".into(),
            ],
        ),
    ] {
        let server =
            Server::configured_globals("127.0.0.1:0", Some(config), None, None, None, &flags)?;
        if server.request("GET", "/health", "")?.0 != 200 {
            return Err("configured HTTP capacity prevented startup".into());
        }
    }
    for (config, flags, field) in [
        (
            json!({"http_max_calls":4,"http_max_connections":2}),
            vec![],
            "http_max_connections",
        ),
        (json!({"http_max_calls":0}), vec![], "http_max_calls"),
        (
            json!({"http_max_connections":0}),
            vec![],
            "http_max_connections",
        ),
        (
            json!({"http_max_connection":4}),
            vec![],
            "http_max_connection",
        ),
        (
            json!({}),
            vec!["--http_max_calls".into(), "9".into()],
            "http_max_connections",
        ),
    ] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        env.config(&config).map_err(|e| e.to_string())?;
        let mut args = vec!["serve".into(), "--http".into(), "127.0.0.1:0".into()];
        args.extend(flags);
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &args)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() || !String::from_utf8_lossy(&output.stdout).contains(field) {
            return Err(format!(
                "startup did not name refused {field}: {}",
                String::from_utf8_lossy(&output.stdout)
            ));
        }
    }
    Ok(())
}
