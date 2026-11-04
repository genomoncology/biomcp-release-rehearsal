//! Match command boundaries without inspecting or changing selector values.
use biomcp_types::ErrorMessage;
use std::ffi::OsString;
#[derive(Default)]
struct Words<'a> {
    positional: Vec<&'a str>,
    source: Option<&'a str>,
    section: Option<&'a str>,
    shared: Option<ErrorMessage>,
    disease: bool,
    manufacturer: bool,
    test_type: bool,
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
        let diagnostic = matches!(
            words.positional.as_slice(),
            ["search" | "get", "diagnostic", ..]
        );
        if diagnostic {
            match flag {
                "-d" | "--disease" => words.disease = true,
                "--manufacturer" => words.manufacturer = true,
                "--type" => words.test_type = true,
                _ => {
                    if let Some(message) = flag_message(flag) {
                        return Some(message);
                    }
                }
            }
        }
        if matches!(
            flag,
            "--section" | "--query" | "--marker" | "--manufacturer" | "--disease" | "-d"
        ) || super::takes_value(root, flag, &words.positional).unwrap_or(false)
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
fn flag_message(flag: &str) -> Option<ErrorMessage> {
    Some(match flag {
        "-g" => ErrorMessage::GeneOption,
        "-t" => ErrorMessage::DiagnosticType,
        "-l" => ErrorMessage::DiseaseLimit,
        "--laboratory" => ErrorMessage::DiagnosticManufacturer,
        "--no-cache" => ErrorMessage::DiseaseFresh,
        "--full" => ErrorMessage::DiagnosticSection,
        "-j" | "--json" => ErrorMessage::DiseaseJson,
        _ => return None,
    })
}
fn guidance(words: Words<'_>) -> Option<ErrorMessage> {
    match words.positional.as_slice() {
        ["gtr", "sync", ..] => return Some(ErrorMessage::DiagnosticGtrSync),
        ["who-ivd", "sync", ..] => return Some(ErrorMessage::DiagnosticWhoSync),
        ["list", "diagnostic", ..] => return Some(ErrorMessage::DiagnosticList),
        _ => (),
    }
    let (verb, rest) = match words.positional.as_slice() {
        [verb @ ("search" | "get"), "diagnostic", rest @ ..] => (*verb, rest),
        _ => return None,
    };
    if words.shared.is_some() {
        return words.shared;
    }
    if words.source.is_none() || words.source == Some("all") {
        return Some(ErrorMessage::DiagnosticRecallSources);
    }
    let section = words.section.or_else(|| rest.get(1).copied());
    if section == Some("all") {
        return Some(ErrorMessage::DiagnosticSection);
    }
    if section == Some("regulatory") && matches!(words.source, Some("gtr" | "who-ivd")) {
        return Some(ErrorMessage::DiagnosticRegulatory);
    }
    if words.disease {
        return Some(if words.source == Some("who-ivd") {
            ErrorMessage::DiagnosticWhoMarker
        } else {
            ErrorMessage::DiagnosticCondition
        });
    }
    if words.manufacturer
        && (verb != "search"
            || !matches!(
                words.source,
                Some("who-ivd" | "fda-device-recall" | "fda-pma" | "fda-510k")
            ))
    {
        return Some(ErrorMessage::DiagnosticManufacturer);
    }
    match (verb, words.source) {
        ("search", Some("who-ivd")) if words.test_type => Some(ErrorMessage::DiagnosticWhoType),
        _ => None,
    }
}
