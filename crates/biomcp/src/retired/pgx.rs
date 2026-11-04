//! PGx migration stays input-free and never rewrites a source query.
use biomcp_types::ErrorMessage;
use std::ffi::OsString;
#[derive(Default)]
struct Words<'a> {
    positional: Vec<&'a str>,
    source: Option<&'a str>,
    section: Option<&'a str>,
    drug_id: bool,
    annotation_id: bool,
    gene: bool,
    shared: Option<ErrorMessage>,
}
pub(super) fn message(root: &clap::Command, args: &[OsString]) -> Option<ErrorMessage> {
    let mut words = Words::default();
    let mut literal = false;
    let mut args = args.iter();
    while let Some(word) = args.next() {
        let word = word.to_str()?;
        if literal || !word.starts_with('-') {
            words.positional.push(word);
            continue;
        }
        if word == "--" {
            literal = true;
            continue;
        }
        let (flag, inline) = word
            .split_once('=')
            .map_or((word, None), |(a, b)| (a, Some(b)));
        if matches!(flag, "-j" | "--json") {
            words.shared = Some(ErrorMessage::DiseaseJson);
        }
        let pgx = matches!(words.positional.as_slice(), ["search" | "get", "pgx", ..]);
        if pgx && let Some(message) = flag_message(flag, words.positional.first().copied()) {
            return Some(message);
        }
        if pgx && flag == "--gene" {
            words.gene = true;
        }
        if pgx && flag == "--annotation-id" {
            words.annotation_id = true;
        }
        if pgx && flag == "--drug-id" {
            words.drug_id = true;
        }
        if matches!(flag, "--section" | "--drug-id")
            || super::takes_value(root, flag, &words.positional).unwrap_or(false)
        {
            let value = inline.or_else(|| args.next().and_then(|v| v.to_str()));
            match flag {
                "--source" => words.source = value,
                "--section" => words.section = value,
                _ => (),
            }
        }
    }
    guidance(words)
}
fn flag_message(flag: &str, verb: Option<&str>) -> Option<ErrorMessage> {
    Some(match flag {
        "-d" | "--drug" | "--evidence" => ErrorMessage::PgxText,
        "--pgx-testing" | "--population" | "--lookup-key" => ErrorMessage::PgxSelector,
        "--full" => ErrorMessage::PgxSection,
        "-l" | "--limit" if verb == Some("get") => ErrorMessage::PgxSectionLimit,
        "-l" => ErrorMessage::DiseaseLimit,
        "--offset" if verb == Some("get") => ErrorMessage::PgxSectionOffset,
        "-j" | "--json" => ErrorMessage::DiseaseJson,
        "--variants" => ErrorMessage::PgxPhase,
        _ => return None,
    })
}
fn guidance(words: Words<'_>) -> Option<ErrorMessage> {
    let (verb, rest) = match words.positional.as_slice() {
        [verb @ ("search" | "get"), "pgx", rest @ ..] => (*verb, rest),
        _ => return None,
    };
    if words.shared.is_some() {
        return words.shared;
    }
    if verb == "search" && !rest.is_empty() {
        return Some(ErrorMessage::PgxGene);
    }
    if rest.len() > 1 && (words.gene || rest.len() > 2) {
        return Some(ErrorMessage::PgxSection);
    }
    let section = words.section.or_else(|| {
        if rest.len() == 2 {
            rest.get(1).copied()
        } else {
            rest.first().copied()
        }
    });
    match section {
        Some("interactions") => return Some(ErrorMessage::PgxInteractions),
        Some("frequencies") => return Some(ErrorMessage::PgxFrequencies),
        Some("all") => return Some(ErrorMessage::PgxSection),
        Some(s)
            if !matches!(
                s,
                "recommendations" | "guidelines" | "diplotype" | "annotations"
            ) =>
        {
            return Some(ErrorMessage::PgxGet);
        }
        _ => (),
    }
    if !words.gene && !words.annotation_id && !rest.is_empty() {
        return Some(ErrorMessage::PgxGet);
    }
    if words.drug_id && (verb != "get" || section != Some("recommendations")) {
        return Some(ErrorMessage::PgxSelector);
    }
    if words.annotation_id {
        return (words.source != Some("clinpgx")).then_some(ErrorMessage::PgxSource);
    }
    match (words.source, verb, section) {
        (Some("cpic"), "search", None) => None,
        (Some("cpic"), "get", None | Some("recommendations" | "guidelines" | "diplotype")) => None,
        (Some("clinpgx"), "get", Some("annotations")) => None,
        _ => Some(ErrorMessage::PgxSource),
    }
}
