//! Existing CLI aliases share one declaration with installed argument replay.
use serde_json::Value;

#[derive(Clone, Copy)]
pub(crate) struct Alias {
    pub(crate) field: &'static str,
    pub(crate) argument: &'static str,
    pub(crate) long: Option<&'static str>,
    pub(crate) short: char,
}

pub(crate) fn declarations(entity: &str, schema: &Value) -> Vec<Alias> {
    let mut aliases = Vec::new();
    let id = schema
        .get("$id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let has = |name| schema.get("properties").and_then(|p| p.get(name)).is_some();
    if id == "biomcp.article-search/3" {
        aliases.push(Alias {
            field: "text",
            argument: "query_alias",
            long: Some("query"),
            short: 'q',
        });
    }
    if entity == "gwas" && has("query") {
        aliases.push(Alias {
            field: "gene",
            argument: "gene",
            long: None,
            short: 'g',
        });
    }
    let short_limit = matches!(entity, "cell-line" | "author" | "gwas") && has("query")
        || entity != "gwas"
            && (id.starts_with("biomcp.pathway-")
                || id.starts_with("biomcp.author-")
                || id == "biomcp.phenotype-search-query/1")
            && has("limit");
    if short_limit {
        aliases.push(Alias {
            field: "limit",
            argument: "limit",
            long: None,
            short: 'l',
        });
    }
    aliases
}
