//! Typed gene search uses the HGNC word index shared with CLI and single MCP.
use super::{
    launch::{self, Environment},
    mcp::{exchange_with_globals, request},
};
use libtest_mimic::Trial;
use serde_json::json;

pub(super) fn trials() -> Vec<Trial> {
    vec![super::transcript_trial(
        "gene-search/gene/names-typed",
        || check().map_err(Into::into),
    )]
}

fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    let replay = launch::fixtures().join("replies/gene/main");
    // The CLI/single-tool case owns name matching and rank assertions. This
    // bridge checks only that typed query mapping reaches the same word index.
    let replies = exchange_with_globals(
        &env,
        &[request(
            json!(1),
            "tools/call",
            crate::runner::protocol::typed_catalog::entity(
                "search_gene",
                json!({"query":"kirsten rat sarcoma"}),
            ),
        )],
        &env.globals(Some(&replay), false),
    )?;
    let value = replies
        .first()
        .and_then(|v| v.pointer("/result/structuredContent"))
        .ok_or_else(|| format!("missing typed gene result: {replies:?}"))?;
    if value.pointer("/status") != Some(&json!("complete"))
        || value.pointer("/page/total") != Some(&json!(1))
        || value.pointer("/result/items/0/symbol") != Some(&json!("KRAS"))
    {
        return Err(format!("typed gene previous-name query: {value}"));
    }
    Ok(())
}
