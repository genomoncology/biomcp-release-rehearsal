#[path = "adapter_values.rs"]
pub(super) mod adapter_values;
#[path = "mapped_refusals.rs"]
mod mapped_refusals;
use super::Check;
use std::{collections::BTreeSet, path::Path};

// The sole dead-code exception contains only the private generated closure include.
const GENERATED_PROTO: &str = "crates/biomcp-adapters/src/alphagenome/proto.rs";

// New exceptions require a reviewed path and reason here.
const EXCEPTIONS: &[(&str, &str)] = &[
    (
        "crates/biomcp-fetch/tests/transport/tls_builds.rs",
        "Count TLS configuration builds on one isolated test thread",
    ),
    (
        "crates/biomcp/src/mcp/http/admission.rs",
        "Bound HTTP framing readers separately from active requests with owned atomic permits",
    ),
    (
        "crates/biomcp-fetch/src/cache/admission.rs",
        "Bind the explicit admission latch to its acquired connection while optional readers probe ownership",
    ),
    (
        "crates/biomcp-fetch/src/grpc/dns_tests.rs",
        "Observe completion of the joined test resolver without shared state outside its fixture",
    ),
    (
        "crates/biomcp-types/src/gwas/search.rs",
        "Serde serialize_with passes a reference to the entire optional field",
    ),
    (
        "crates/biomcp-fetch/src/secret.rs",
        "Resolve one deferred credential once per configured secret",
    ),
    (
        "crates/biomcp-fetch/src/cancellation.rs",
        "Cancel all request workers through one command-owned atomic flag",
    ),
    (
        "crates/biomcp-fetch/src/grpc/control.rs",
        "Acknowledge stream cancellation before releasing the worker permit",
    ),
    (
        "crates/biomcp-fetch/src/grpc/worker.rs",
        "Serialize lazy runtime initialization and join its sole worker thread",
    ),
    (
        GENERATED_PROTO,
        "Retain the complete pinned generated provider import closure",
    ),
    (
        "crates/biomcp-adapters/src/monarch/native_contracts.rs",
        "Locate and read retained external bodies only inside the ignored private mapper contract",
    ),
    (
        "crates/biomcp-adapters/src/registry.rs",
        "Require every source outcome to use its unique publisher declaration",
    ),
    (
        "crates/biomcp-types/src/source.rs",
        "Convert closed nonempty source spellings into neutral provenance text",
    ),
    (
        "crates/biomcp/src/mcp/http.rs",
        "Own bounded HTTP connection workers and join completed handles",
    ),
    (
        "crates/biomcp/src/mcp/http/call.rs",
        "Own and join bounded hosted HTTP tool workers through timeout and shutdown",
    ),
    (
        "crates/biomcp-adapters/src/pmc_oa/supplement.rs",
        "Retain one parsed binary reply across its checked cache admission callback",
    ),
    (
        "crates/biomcp-fetch/src/observe.rs",
        "Serialize bounded opt-in attempt summaries across transport workers",
    ),
    (
        "crates/biomcp-query/src/store/mod.rs",
        "Own SQLite connections and wall-clock pool timestamps",
    ),
    (
        "crates/biomcp-query/src/fan_out.rs",
        "Join scoped source jobs before returning the command result",
    ),
    (
        "crates/biomcp-fetch/src/batch.rs",
        "Own bounded scoped callback threads and join their clocks before returning",
    ),
    (
        "crates/biomcp-fetch/src/deadline.rs",
        "Advance shared replay time without real sleeping",
    ),
    (
        "crates/biomcp-fetch/src/record.rs",
        "Reserve shared replay cursors and publish complete captures",
    ),
    (
        "crates/biomcp/src/mcp.rs",
        "Own bounded stdio input and tool threads",
    ),
    (
        "crates/biomcp-fetch/src/lib.rs",
        "Retain typed leg reports across concurrent scoped requests",
    ),
    (
        "crates/biomcp-fetch/src/cache.rs",
        "Serialize SQLite operations on the shared transport connection",
    ),
    (
        "crates/biomcp/src/config.rs",
        "Read process arguments at the command boundary",
    ),
    (
        "crates/biomcp-fetch/src/quota/limiter.rs",
        "Shared transport quota state and FIFO waits",
    ),
];
const FILE_LIMIT: usize = 600;
const LINE_WIDTH: usize = 160;
const ADAPTER_LIMIT: usize = 300;

#[path = "source_files.rs"]
mod files;

pub(super) fn check(root: &Path) -> Check {
    super::diagnostics(vec![
        super::placement::check(root),
        adapter_values::check(root),
        mapped_refusals::check(root),
        super::transport_sources::check(root),
        files::check(root),
    ])
}

fn width(path: &str, text: &str) -> Check {
    for (index, line) in text.lines().enumerate() {
        if line.chars().count() > LINE_WIDTH {
            return fail(
                path,
                &format!("line width limit exceeded at line {}", index + 1),
            );
        }
    }
    Ok(())
}

fn fail(path: &str, rule: &str) -> Check {
    Err(format!("{path}: {rule}").into())
}

fn attributes(root: &Path, path: &str, tokens: &[String]) -> Check {
    for (i, token) in tokens.iter().enumerate() {
        if token != "#" {
            continue;
        }
        let tail = tokens.get(i + 1..).unwrap_or_default();
        let tail = tail.strip_prefix(&["!".to_owned()]).unwrap_or(tail);
        if tail.first().is_none_or(|v| v != "[") {
            continue;
        }
        let attribute: Vec<_> = tail
            .iter()
            .take_while(|v| v.as_str() != "]")
            .map(String::as_str)
            .collect();
        let suppression = attribute.iter().any(|v| matches!(*v, "allow" | "expect"));
        if attribute.contains(&"allow") {
            return fail(
                path,
                "allow attributes forbidden; use a reasoned expectation",
            );
        }
        if tokens.get(i + 1).is_some_and(|v| v == "!") && attribute.contains(&"disallowed_types") {
            return fail(
                path,
                "interior state expectations belong on the owning item",
            );
        }
        if suppression
            && (attribute.contains(&"unused_imports")
                || attribute.contains(&"dead_code") && path != GENERATED_PROTO)
        {
            return fail(path, "dead_code and unused_imports suppressions forbidden");
        }
        if suppression
            && (!EXCEPTIONS.iter().any(|(file, _)| *file == path) || !attribute.contains(&"reason"))
        {
            return fail(path, "lint exception requires listed location and reason");
        }
        if path == GENERATED_PROTO && suppression {
            let after = tail
                .iter()
                .position(|v| v == "]")
                .and_then(|i| tail.get(i + 1..));
            let only_include = ["include", "!", "(", "\"generated/mod.rs\"", ")", ";"];
            let only_dead_code = matches!(
                attribute.as_slice(),
                ["[", "expect", "(", "dead_code", ",", "reason", "=", _, ")"]
                    | [
                        "[",
                        "expect",
                        "(",
                        "dead_code",
                        ",",
                        "reason",
                        "=",
                        _,
                        ",",
                        ")"
                    ]
            );
            if !only_dead_code
                || i != 0
                || tokens.get(i + 1).is_none_or(|v| v != "!")
                || after.is_none_or(|v| v.iter().map(String::as_str).ne(only_include))
            {
                return fail(
                    path,
                    "generated exception requires only the private closure include",
                );
            }
        }
        if path.starts_with("crates/biomcp-adapters/src/")
            && let Some(site) = adapter_values::attribute_site(tokens, i)
            && !adapter_values::private_refusal::allowed(root, path, tokens, i)
            && !adapter_values::pending(path, &site, 1)?
        {
            return fail(path, "raw serde default or deny_unknown_fields forbidden");
        }
    }
    Ok(())
}

fn references(
    root: &Path,
    path: &str,
    text: &str,
    tokens: &[String],
    sqlite: &[String],
    keys: &mut BTreeSet<String>,
) -> Check {
    if path.starts_with("crates/biomcp-query/src/")
        && !path.starts_with("crates/biomcp-query/src/store/")
        && tokens.iter().any(|v| sqlite.contains(v))
    {
        return fail(path, "SQLite references belong in src/store/");
    }
    for line in text.lines() {
        if let Some((_, reference)) = line.split_once("// Quirk:") {
            let case = reference
                .split_whitespace()
                .find(|part| part.ends_with(".yaml"));
            if case.is_none_or(|case| {
                !Path::new(case)
                    .components()
                    .all(|part| matches!(part, std::path::Component::Normal(_)))
                    || !root.join("tests/cases").join(case).is_file()
            }) {
                return fail(path, "Quirk requires an existing case path");
            }
        }
    }
    let mut source_keys = vec!["SourceKey".to_owned()];
    let mut quotas = vec!["Quota".to_owned()];
    for alias in tokens.windows(3) {
        if alias.get(1).is_some_and(|v| v == "as") {
            if alias.first().is_some_and(|v| source_keys.contains(v)) {
                source_keys.push(alias.get(2).cloned().unwrap_or_default());
            }
            if alias.first().is_some_and(|v| quotas.contains(v)) {
                quotas.push(alias.get(2).cloned().unwrap_or_default());
            }
        }
    }
    for (i, token) in tokens.iter().enumerate() {
        let tail = tokens.get(i + 1..).unwrap_or_default();
        let declaration = source_keys.contains(token)
            && tail.starts_with(&[":".into(), ":".into(), "declare".into(), "(".into()]);
        let quota = token == "static"
            && i.checked_sub(1)
                .and_then(|i| tokens.get(i))
                .is_none_or(|v| v != "\'")
            && tail
                .iter()
                .take_while(|v| v.as_str() != "=")
                .any(|v| quotas.contains(v));
        if (declaration || quota) && path != "crates/biomcp-adapters/src/quotas.rs" {
            return fail(path, "quota declarations belong in adapter quotas.rs");
        }
        if declaration {
            let Some(key) = tail.get(4).filter(|v| v.starts_with('"')) else {
                return fail(path, "source key declaration requires a literal key");
            };
            if !keys.insert(key.clone()) {
                return fail(path, "duplicate source key declaration");
            }
        }
    }
    Ok(())
}

// Only identifiers, literals, comments and punctuation are needed; no Rust AST.
pub(super) fn tokens(text: &str) -> Vec<String> {
    let mut input = text.chars().peekable();
    let mut result = Vec::new();
    while let Some(ch) = input.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '/' && input.peek() == Some(&'/') {
            for ch in input.by_ref() {
                if ch == '\n' {
                    break;
                }
            }
        } else if ch == '/' && input.peek() == Some(&'*') {
            input.next();
            let mut depth = 1;
            while let Some(ch) = input.next() {
                if ch == '/' && input.peek() == Some(&'*') {
                    input.next();
                    depth += 1;
                }
                if ch == '*' && input.peek() == Some(&'/') {
                    input.next();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
        } else if ch == '\''
            && let Some(length) = character_end(input.clone())
        {
            let value: String = std::iter::once(ch)
                .chain(input.by_ref().take(length))
                .collect();
            result.push(value);
        } else if ch == '"' {
            let mut value = String::from(ch);
            while let Some(ch) = input.next() {
                value.push(ch);
                if ch == '\\' {
                    if let Some(ch) = input.next() {
                        value.push(ch);
                    }
                } else if ch == '"' {
                    break;
                }
            }
            result.push(value);
        } else if ch.is_alphanumeric() || ch == '_' {
            let mut value = String::from(ch);
            while input
                .peek()
                .is_some_and(|ch| ch.is_alphanumeric() || *ch == '_')
            {
                if let Some(ch) = input.next() {
                    value.push(ch);
                }
            }
            // Raw strings may contain quotes, comments and item delimiters.
            if matches!(value.as_str(), "r" | "br" | "cr") {
                let hashes = input.clone().take_while(|ch| *ch == '#').count();
                if input.clone().nth(hashes) == Some('"') {
                    input.by_ref().take(hashes + 1).for_each(drop);
                    value = String::from("\"");
                    while let Some(ch) = input.next() {
                        value.push(ch);
                        if ch == '"' && input.clone().take(hashes).all(|ch| ch == '#') {
                            input.by_ref().take(hashes).for_each(drop);
                            break;
                        }
                    }
                }
            }
            result.push(value);
        } else {
            result.push(ch.to_string());
        }
    }
    result
}

// A lifetime has no closing quote after one character or escape.
fn character_end(mut input: std::iter::Peekable<std::str::Chars<'_>>) -> Option<usize> {
    let mut length = 1;
    if input.next()? == '\\' {
        length += 1;
        match input.next()? {
            'u' if input.next()? == '{' => {
                length += 1;
                loop {
                    length += 1;
                    if input.next()? == '}' {
                        break;
                    }
                }
            }
            'x' => {
                input.next()?;
                input.next()?;
                length += 2;
            }
            _ => {}
        }
    }
    (input.next()? == '\'').then_some(length + 1)
}

fn adapter_helpers(path: &str, tokens: &[String]) -> Check {
    if !path.starts_with("crates/biomcp-adapters/src/")
        && !path.starts_with("crates/biomcp-fetch/src/")
        && !path.starts_with("crates/biomcp-query/src/")
    {
        return Ok(());
    }
    for pair in tokens.windows(2) {
        let [word, name] = pair else {
            continue;
        };
        if word != "fn" {
            continue;
        }
        let home = match name.as_str() {
            "bad" => Some(""),
            "bad_reply" => Some("crates/biomcp-fetch/src/error.rs"),
            "received_date" => Some("crates/biomcp-adapters/src/received.rs"),
            "root" => Some("crates/biomcp-adapters/src/xml_element.rs"),
            _ => None,
        };
        if home.is_some_and(|home| path != home) {
            return fail(path, "copied helper must use its shared home");
        }
    }

    if !path.starts_with("crates/biomcp-adapters/src/")
        || path == "crates/biomcp-adapters/src/mapped.rs"
    {
        return Ok(());
    }
    // These decode or render text formats; they do not construct neutral Text values.
    let format_text = matches!(
        path,
        "crates/biomcp-adapters/src/xml_element.rs"
            | "crates/biomcp-adapters/src/jats/markdown.rs"
            | "crates/biomcp-adapters/src/sheet/xlsx.rs"
    );
    if tokens
        .windows(2)
        .any(|pair| matches!(pair, [word, name] if word == "fn" && (name == "problem" || name == "text" && !format_text)))
    {
        return fail(path, "adapter mapping helpers belong in mapped.rs");
    }
    Ok(())
}

fn tool_modes(relative: &str, text: &str) -> Check {
    if matches!(
        relative,
        "crates/biomcp/tests/cases.rs" | "crates/biomcp/tests/runner/case_main.rs"
    ) && tokens(text)
        .iter()
        .filter_map(|v| v.strip_prefix("\"--"))
        .filter_map(|v| v.strip_suffix('"'))
        .any(|flag| flag != "cases" && !flag.contains(char::is_whitespace))
    {
        return fail(relative, "tool modes belong in tools.rs");
    }
    Ok(())
}
