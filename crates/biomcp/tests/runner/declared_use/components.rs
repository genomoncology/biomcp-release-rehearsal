//! Feature mcp-hosted: registry component restrictions follow each hosted request.
use super::{ACADEMIC, call};
use crate::runner::{http::Server, launch};
use serde_json::{Value, json};

pub(super) fn check() -> Result<(), String> {
    for header in [ACADEMIC, "", "x-biomcp-use: commercial\r\n"] {
        let mut server = Server::configured_globals(
            "127.0.0.1:0",
            None,
            Some(launch::fixtures().join("replies/variant/synthetic-components")),
            None,
            None,
            &["--hosted".into()],
        )?;
        let reply = call(
            &server,
            &[
                "get",
                "variant",
                "rs113488022",
                "predict",
                "conservation",
                "cosmic",
            ],
            header,
            false,
        )?;
        let candidates = reply
            .pointer("/result/structuredContent/result/candidates")
            .and_then(Value::as_array)
            .filter(|rows| !rows.is_empty())
            .ok_or_else(|| format!("component candidates missing: {reply}"))?;
        for candidate in candidates {
            for section in ["predict", "conservation", "cosmic"] {
                let expected = if header == ACADEMIC {
                    "data"
                } else {
                    "skipped"
                };
                if candidate.pointer(&format!("/sections/{section}/state"))
                    != Some(&json!(expected))
                {
                    return Err(format!("component did not follow declared use: {reply}"));
                }
                if header != ACADEMIC
                    && candidate.pointer(&format!("/sections/{section}/reason"))
                        != Some(&json!("disabled"))
                {
                    return Err(format!(
                        "commercial component did not retain disabled refusal: {reply}"
                    ));
                }
            }
        }
        server.stop()?;
    }
    Ok(())
}
