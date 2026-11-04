//! Fixed adverse-event migration guidance before command parsing and I/O.
use biomcp_types::ErrorMessage;
use std::ffi::OsString;
const FLAGS: &[(&str, ErrorMessage)] = &[
    ("-d", ErrorMessage::AdverseText),
    ("--device", ErrorMessage::AdverseMaude),
    ("--outcome", ErrorMessage::AdverseOutcome),
    ("--date-from", ErrorMessage::AdverseFrom),
    ("--since", ErrorMessage::AdverseFrom),
    ("--date-to", ErrorMessage::AdverseTo),
    ("--until", ErrorMessage::AdverseTo),
    ("--suspect-only", ErrorMessage::AdverseFilter),
    ("--sex", ErrorMessage::AdverseFilter),
    ("--age-min", ErrorMessage::AdverseFilter),
    ("--age-max", ErrorMessage::AdverseFilter),
    ("--reporter", ErrorMessage::AdverseFilter),
    ("--count", ErrorMessage::AdverseFilter),
    ("--classification", ErrorMessage::AdverseDeviceRecalls),
    ("-l", ErrorMessage::DiseaseLimit),
    ("-j", ErrorMessage::DiseaseJson),
    ("--json", ErrorMessage::DiseaseJson),
];
pub(super) fn message(root: &clap::Command, args: &[OsString]) -> Option<ErrorMessage> {
    let mut positional = Vec::new();
    let mut source = None;
    let mut serious = None;
    let mut faers_filter = false;
    let mut maude_filter = false;
    let mut section = None;
    let mut shared = None;
    let mut literal = false;
    let mut args = args.iter();
    while let Some(word) = args.next() {
        let word = word.to_str()?;
        if literal || !word.starts_with('-') {
            positional.push(word);
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
            shared = Some(ErrorMessage::DiseaseJson);
        }
        let adverse = matches!(
            positional.as_slice(),
            ["search" | "get", "adverse-event", ..]
        );
        if adverse
            && matches!(
                flag,
                "--drug" | "--reaction" | "--serious" | "--received-from" | "--received-to"
            )
        {
            faers_filter = true;
        }
        if adverse && matches!(flag, "--manufacturer" | "--product-code" | "--problem") {
            maude_filter = true;
        }
        if adverse && let Some(found) = super::find(FLAGS, flag) {
            return Some(found);
        }
        if adverse && flag == "--version" && positional.first() == Some(&"get") {
            return Some(ErrorMessage::AdverseVersion);
        }
        if adverse && flag == "--type" {
            return match inline.or_else(|| args.next().and_then(|v| v.to_str())) {
                Some("faers") => Some(ErrorMessage::AdverseFaers),
                Some("device" | "devices" | "maude") => Some(ErrorMessage::AdverseMaude),
                Some("recall" | "recalls" | "enforcement") => {
                    Some(ErrorMessage::AdverseDeviceRecalls)
                }
                _ => None,
            };
        }
        if adverse && flag == "--serious" {
            serious = Some(inline.or_else(|| args.next().and_then(|v| v.to_str())));
        } else if flag == "--section"
            || super::takes_value(root, flag, &positional).unwrap_or(false)
        {
            let value = inline.or_else(|| args.next().and_then(|v| v.to_str()));
            match flag {
                "--source" => source = value,
                "--section" => section = value,
                _ => (),
            }
        }
    }
    if maude_filter && source != Some("maude") {
        return Some(ErrorMessage::AdverseFilter);
    }
    if source == Some("vaers") && faers_filter {
        return Some(ErrorMessage::VaersFilter);
    }
    guidance(&positional, source, serious, section, shared, literal)
}
fn guidance(
    positional: &[&str],
    source: Option<&str>,
    serious: Option<Option<&str>>,
    section: Option<&str>,
    shared: Option<ErrorMessage>,
    literal: bool,
) -> Option<ErrorMessage> {
    match positional {
        ["drug", "adverse-events", ..] => Some(ErrorMessage::AdverseText),
        ["search" | "get", "adverse-event", rest @ ..] => {
            if source == Some("cvx") {
                return Some(ErrorMessage::CvxUnavailable);
            }
            if source == Some("vaers") {
                if positional.first() == Some(&"get") {
                    return Some(ErrorMessage::VaersGet);
                }
                let text = rest.first().copied().unwrap_or("");
                if text.eq_ignore_ascii_case("FLU")
                    || text.eq_ignore_ascii_case("influenza")
                    || text.eq_ignore_ascii_case("Influenza vaccine")
                {
                    return Some(ErrorMessage::VaersInfluenzaCodes);
                }
                if biomcp_types::vaers_vaccine_code(text).is_none() {
                    return Some(ErrorMessage::VaersVaccineChoices);
                }
                if serious.is_some() {
                    return Some(ErrorMessage::VaersFilter);
                }
            }
            if shared.is_some() {
                return shared;
            }
            if source == Some("all") {
                return Some(ErrorMessage::AdverseSource);
            }
            if let Some(value) = serious {
                if source == Some("maude") {
                    return Some(ErrorMessage::AdverseFilter);
                }
                if !matches!(
                    value,
                    Some(
                        "any"
                            | "death"
                            | "hospitalization"
                            | "lifethreatening"
                            | "disabling"
                            | "congenital-anomaly"
                            | "other"
                    )
                ) {
                    return Some(ErrorMessage::AdverseSeriousChoices);
                }
            }
            if positional.first() == Some(&"get") {
                let selected =
                    section.or_else(|| if literal { None } else { rest.get(1).copied() });
                return match selected {
                    Some("outcomes") => Some(ErrorMessage::AdverseOutcomes),
                    Some("concomitant") => Some(ErrorMessage::AdverseConcomitant),
                    Some("guidance" | "all") => Some(ErrorMessage::AdverseSection),
                    _ => None,
                };
            }
            None
        }
        _ => None,
    }
}
