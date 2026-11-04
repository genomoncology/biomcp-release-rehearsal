//! Bind adapter policies to their source declarations and exhaustive dispatcher.
use super::{Check, source::tokens};
use std::{collections::BTreeMap, fs, path::Path};

struct Policy {
    path: String,
    source: String,
}
struct Declaration {
    path: String,
    name: String,
    spec: Vec<String>,
    auxiliary: Vec<String>,
}

pub(super) fn check(root: &Path) -> Check {
    let adapters = root.join("crates/biomcp-adapters/src");
    if !adapters.exists() {
        return Ok(());
    }
    let mut policies = BTreeMap::new();
    let mut declarations = Vec::new();
    for path in super::rust_files(&adapters)? {
        let relative = path
            .strip_prefix(&adapters)?
            .to_string_lossy()
            .replace('\\', "/");
        let input = tokens(&fs::read_to_string(&path)?);
        collect(
            &format!("crates/biomcp-adapters/src/{relative}"),
            &input,
            &mut policies,
            &mut declarations,
        )?;
    }
    let dispatcher = fs::read_to_string(adapters.join("decl.rs")).unwrap_or_default();
    let dispatcher = tokens(&dispatcher).join("");
    let mut registered = BTreeMap::new();
    for declaration in declarations {
        let source = dispatched_source(&dispatcher, &declaration)?;
        let primary = references(&declaration.spec, &declaration.path);
        if declaration
            .spec
            .iter()
            .any(|token| token == "component_spec")
        {
            if constructor_source(&declaration.spec).as_deref() != Some(source.as_str()) {
                return Err(format!(
                    "{}: source declaration references the wrong source",
                    declaration.path
                )
                .into());
            }
        } else {
            if primary.len() != 1 {
                return Err(format!(
                    "{}: source declaration requires one exact policy reference",
                    declaration.path
                )
                .into());
            }
            register(
                &declaration.path,
                &source,
                primary,
                &policies,
                &mut registered,
            )?;
        }
        register(
            &declaration.path,
            &source,
            references(&declaration.auxiliary, &declaration.path),
            &policies,
            &mut registered,
        )?;
    }
    for (reference, policy) in policies {
        if !registered.contains_key(&reference) {
            return Err(format!(
                "{}: implemented source must appear in its SourceDecl: expected {reference}",
                policy.path
            )
            .into());
        }
    }
    Ok(())
}

fn dispatched_source(
    dispatcher: &str,
    declaration: &Declaration,
) -> Result<String, Box<dyn std::error::Error>> {
    let target = format!(
        "&crate::{}::{}",
        module(&declaration.path),
        declaration.name
    );
    dispatcher
        .split(',')
        .find_map(|arm| {
            let (before, after) = arm.split_once("=>")?;
            (after == target).then(|| before.rsplit("Source::").next().unwrap_or("").to_owned())
        })
        .ok_or_else(|| {
            format!(
                "{}: source declaration must appear in decl(source): {target}",
                declaration.path
            )
            .into()
        })
}

fn return_type(prefix: &[String]) -> bool {
    prefix
        .rsplit(|token| matches!(token.as_str(), "{" | "}" | ";"))
        .next()
        .unwrap_or_default()
        .windows(2)
        .any(|pair| pair == ["-", ">"])
}

fn collect(
    path: &str,
    input: &[String],
    policies: &mut BTreeMap<String, Policy>,
    declarations: &mut Vec<Declaration>,
) -> Check {
    for (i, token) in input.iter().enumerate() {
        if input.get(i + 1).is_none_or(|token| token != "{")
            || return_type(input.get(..i).unwrap_or_default())
            || input
                .get(i.wrapping_sub(1))
                .is_some_and(|token| token == "struct")
        {
            continue;
        }
        let body = body(input.get(i + 2..).unwrap_or_default());
        if token == "Terms" {
            terms(path, body)?;
        } else if token == "SourceSpec" {
            if field(body, "terms").is_none() {
                return Err(format!("{path}: source declaration requires publisher terms").into());
            }
            if generic_constructor(path, input.get(..i).unwrap_or_default(), body) {
                continue;
            }
            let name = named(input.get(..i).unwrap_or_default(), "static")
                .ok_or_else(|| format!("{path}: source declarations require named statics"))?;
            policies.insert(
                format!("&crate::{}::{name}", module(path)),
                Policy {
                    path: path.into(),
                    source: source_variant(field(body, "source").unwrap_or_default()).ok_or_else(
                        || {
                            format!(
                                "{path}: concrete source policy requires an exact Source variant"
                            )
                        },
                    )?,
                },
            );
        } else if token == "SourceDecl" {
            let name = named(input.get(..i).unwrap_or_default(), "static")
                .ok_or_else(|| format!("{path}: source declarations require named statics"))?;
            declarations.push(Declaration {
                path: path.into(),
                name,
                spec: field(body, "spec").unwrap_or_default().to_vec(),
                auxiliary: field(body, "auxiliary").unwrap_or_default().to_vec(),
            });
        }
    }
    Ok(())
}

fn register(
    path: &str,
    source: &str,
    references: Vec<String>,
    policies: &BTreeMap<String, Policy>,
    registered: &mut BTreeMap<String, String>,
) -> Check {
    for reference in references {
        let policy = policies.get(&reference).ok_or_else(|| {
            format!("{path}: source declaration references an unknown policy: {reference}")
        })?;
        if policy.source != source {
            return Err(format!(
                "{path}: source declaration references the wrong source: {reference}"
            )
            .into());
        }
        if registered.insert(reference.clone(), path.into()).is_some() {
            return Err(format!(
                "{path}: source policy has more than one declaration owner: {reference}"
            )
            .into());
        }
    }
    Ok(())
}

fn terms(path: &str, input: &[String]) -> Check {
    for name in ["name", "url", "usage"] {
        let value = field(input, name).unwrap_or_default();
        if value.is_empty() || name != "usage" && value == ["\"\""] {
            return Err(format!("{path}: source declaration requires publisher terms").into());
        }
    }
    Ok(())
}

fn named(input: &[String], keyword: &str) -> Option<String> {
    let i = input.iter().rposition(|token| token == keyword)?;
    let suffix = input.get(i..)?;
    (!suffix
        .iter()
        .any(|token| matches!(token.as_str(), ";" | "{" | "}")))
    .then(|| suffix.get(1).cloned())
    .flatten()
}

fn module(path: &str) -> String {
    path.strip_prefix("crates/biomcp-adapters/src/")
        .unwrap_or(path)
        .trim_end_matches(".rs")
        .trim_end_matches("/mod")
        .replace('/', "::")
}

fn source_variant(input: &[String]) -> Option<String> {
    let value = input.join("");
    let value = value.strip_prefix("biomcp_types::").unwrap_or(&value);
    value
        .strip_prefix("Source::")
        .filter(|variant| {
            !variant.is_empty() && variant.chars().all(|c| c.is_alphanumeric() || c == '_')
        })
        .map(str::to_owned)
}

fn constructor_source(input: &[String]) -> Option<String> {
    let start = input.iter().position(|token| token == "(")? + 1;
    let rest = input.get(start..)?;
    let end = rest.iter().position(|token| token == ",")?;
    source_variant(rest.get(..end)?)
}

fn generic_constructor(path: &str, prefix: &[String], input: &[String]) -> bool {
    let function = prefix
        .iter()
        .rposition(|token| token == "fn")
        .and_then(|i| prefix.get(i + 1));
    if path != "crates/biomcp-adapters/src/decl.rs"
        || function.is_none_or(|name| name != "component_spec")
    {
        return false;
    }
    // Only the reviewed argument-preserving constructor may build an unnamed policy.
    // Every field is fixed here; literals or computed provider facts require named statics.
    let expected = [
        ("source", "source"),
        ("terms", "terms"),
        ("base", "carrier.base"),
        ("destinations", "biomcp_fetch::Destinations::Primary"),
        ("user_agent", "carrier.user_agent"),
        ("key", "KeyUse::None"),
        ("quota", "carrier.quota"),
        ("deadline", "carrier.deadline"),
    ];
    expected
        .iter()
        .all(|(name, value)| field(input, name).is_some_and(|actual| actual.join("") == *value))
        && !input.iter().any(|token| token.starts_with('"'))
}

fn references(input: &[String], path: &str) -> Vec<String> {
    let parent = module(path)
        .rsplit_once("::")
        .map_or(String::new(), |(parent, _)| parent.to_owned());
    let mut references = Vec::new();
    for (i, token) in input.iter().enumerate() {
        if token != "&"
            || input
                .get(i + 1)
                .is_none_or(|token| !matches!(token.as_str(), "crate" | "super"))
        {
            continue;
        }
        let value: String = input
            .get(i..)
            .unwrap_or_default()
            .iter()
            .take_while(|token| {
                token.as_str() == "&"
                    || token.as_str() == ":"
                    || token.chars().all(|c| c.is_alphanumeric() || c == '_')
            })
            .cloned()
            .collect();
        references.push(value.replacen("&super::", &format!("&crate::{parent}::"), 1));
    }
    references
}

fn body(input: &[String]) -> &[String] {
    let mut depth = 1;
    let end = input
        .iter()
        .position(|token| {
            if token == "{" {
                depth += 1;
            }
            if token == "}" {
                depth -= 1;
            }
            depth == 0
        })
        .unwrap_or(input.len());
    input.get(..end).unwrap_or_default()
}

fn field<'a>(input: &'a [String], name: &str) -> Option<&'a [String]> {
    let mut depth = 0;
    let mut start = 0;
    for (i, token) in input.iter().enumerate() {
        if matches!(token.as_str(), "{" | "[" | "(") {
            depth += 1;
        }
        if matches!(token.as_str(), "}" | "]" | ")") {
            depth -= 1;
        }
        if token == "," && depth == 0 {
            if input.get(start).is_some_and(|token| token == name) {
                let value = if input.get(start + 1).is_some_and(|token| token == ":") {
                    start + 2
                } else {
                    start
                };
                return input.get(value..i);
            }
            start = i + 1;
        }
    }
    if input.get(start).is_some_and(|token| token == name) {
        let value = if input.get(start + 1).is_some_and(|token| token == ":") {
            start + 2
        } else {
            start
        };
        input.get(value..)
    } else {
        None
    }
}
