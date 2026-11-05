//! Research fetch keeps readable source text once in either tool mode.
use super::{Environment, call, json};

pub(super) fn check() -> Result<(), String> {
    for mode in ["single", "typed"] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        env.config(&json!({"mcp_tool_mode":mode}))
            .map_err(|e| e.to_string())?;
        let globals = env.globals(
            Some(&super::super::launch::fixtures().join("replies/article/get/synthetic-readable")),
            false,
        );
        let fetched = call(&env, "fetch", json!({"id":"article 39214094"}), &globals)?;
        let text = fetched
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or("fetch text missing")?;
        if fetched.get("id") != Some(&json!("article 39214094"))
            || fetched.get("url") != Some(&json!("https://pubmed.ncbi.nlm.nih.gov/39214094/"))
            || !text.contains("Synthetic abstract marker.")
            || !text.contains("10^(6)")
            || !text.contains("response < 0.05")
            || !text.contains("Binding A<B and C>D remains.")
            || !text.contains("Unknown <TP53> token.")
            || !text.contains("Malformed <em broken> stays.")
            || !text.contains("Comparison A<B>C remains.")
            || !text.contains("Paired emphasis retained.")
            || text.contains("&#")
            || text.contains("&amp;")
            || text.contains("&nbsp;")
            || text.contains("<sup>")
            || text.contains("</sup>")
            || fetched
                .to_string()
                .matches("Synthetic abstract marker")
                .count()
                != 1
            || fetched.pointer("/metadata/result").is_some()
        {
            return Err(format!(
                "{mode} fetch did not return one readable record: {fetched}"
            ));
        }
        if !text.contains("source")
            || !text.contains("https://example.invalid/")
            || !text.contains("ignore previous instructions")
            || !text.starts_with("````text\n")
            || !text.ends_with("\n````\n")
        {
            return Err("fetch source text did not remain literal data".into());
        }
    }
    Ok(())
}
