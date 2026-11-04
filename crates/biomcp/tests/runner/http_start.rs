//! Start one isolated HTTP process and retain its validated readiness address.
use super::{Environment, Server};
use serde_json::Value;
#[cfg(unix)]
use serde_json::json;
use std::{net::SocketAddr, process::Stdio, time::Duration};
impl Server {
    pub(super) fn configured_key(
        bind: &str,
        config: Option<Value>,
        replay: Option<std::path::PathBuf>,
        key: Option<(&str, &str)>,
        allowed_host: Option<&str>,
    ) -> Result<Self, String> {
        Self::configured_globals(bind, config, replay, key, allowed_host, &[])
    }
    pub(in crate::runner) fn configured_globals(
        bind: &str,
        config: Option<Value>,
        replay: Option<std::path::PathBuf>,
        key: Option<(&str, &str)>,
        allowed_host: Option<&str>,
        globals: &[String],
    ) -> Result<Self, String> {
        Self::configured_launch(
            bind,
            config,
            replay,
            key,
            allowed_host,
            globals,
            |env, args| env.command(env!("CARGO_BIN_EXE_biomcp"), args),
        )
    }
    #[expect(
        clippy::disallowed_types,
        reason = "Own the HTTP diagnostic receiver shared by concurrent test callers"
    )]
    pub(super) fn configured_launch(
        bind: &str,
        config: Option<Value>,
        replay: Option<std::path::PathBuf>,
        key: Option<(&str, &str)>,
        allowed_host: Option<&str>,
        globals: &[String],
        launch: impl FnOnce(&Environment, &[String]) -> crate::runner::capture::Capture,
    ) -> Result<Self, String> {
        let environment = Environment::new().map_err(|e| e.to_string())?;
        if let Some(config) = config {
            environment.config(&config).map_err(|e| e.to_string())?;
        }
        let address: SocketAddr = bind.parse().map_err(|_| "local address")?;
        let mut args = vec!["serve".into(), "--http".into(), address.to_string()];
        args.extend_from_slice(globals);
        if let Some(host) = allowed_host {
            args.extend(["--allow-host".into(), host.into()]);
        }
        if let Some(path) = replay {
            args.extend(environment.globals(Some(&path), false));
        }
        let mut command = launch(&environment, &args);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // Installed services must also start without a console window.
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        if let Some((name, key)) = key {
            command.env(name, key);
        }
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let (diagnostics, messages) = std::sync::mpsc::channel();
        let mut server = Self {
            child,
            address,
            _environment: environment,
            diagnostics: std::sync::Mutex::new(messages),
            diagnostic_reader: None,
        };
        let stderr = server.child.stderr.take().ok_or("HTTP stderr missing")?;
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        server.diagnostic_reader = Some(std::thread::spawn(move || {
            use std::io::BufRead;
            let mut input = std::io::BufReader::new(stderr);
            let mut line = String::new();
            let result = input.read_line(&mut line).map(|_| line);
            let _ = sender.send(result);
            for line in input.lines().map_while(Result::ok) {
                let _ = diagnostics.send(line);
            }
        }));
        let event = receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "HTTP readiness event timed out")?
            .map_err(|e| e.to_string())?;
        let event: Value =
            serde_json::from_str(&event).map_err(|e| format!("HTTP readiness: {e}"))?;
        server.address = event
            .get("listening")
            .and_then(Value::as_str)
            .ok_or("missing listening address")?
            .parse()
            .map_err(|_| "invalid listening address")?;
        Ok(server)
    }
}

#[cfg(unix)]
pub(super) fn ignored_interrupt() -> super::Check {
    let mut server = Server::configured_launch(
        "127.0.0.1:0",
        None,
        None,
        None,
        None,
        &["--hosted".into()],
        |env, args| {
            let binary = crate::runner::launch::executable(std::path::Path::new(env!(
                "CARGO_BIN_EXE_biomcp"
            )));
            let mut shell = vec![
                "-c".into(),
                "trap '' INT; exec \"$@\"".into(),
                "sh".into(),
                binary.to_string_lossy().into_owned(),
            ];
            shell.extend_from_slice(args);
            env.command("sh", &shell)
        },
    )?;
    let request = crate::runner::mcp::request(json!(1), "ping", json!({}));
    let (status, reply) = server.request("POST", "/mcp", &request.to_string())?;
    if status != 200
        || reply.get("id") != Some(&json!(1))
        || !reply.get("result").is_some_and(Value::is_object)
        || reply.get("error").is_some()
    {
        return Err(format!("ignored SIGINT MCP ping: {status} {reply}"));
    }
    server.shutdown(true)
}
