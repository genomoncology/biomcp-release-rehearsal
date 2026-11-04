//! Disease migration is fixed guidance, never a source request or query rewrite.
use biomcp_types::ErrorMessage;
use std::ffi::OsString;
const FLAGS: &[(&str, ErrorMessage)] = &[
    ("-q", ErrorMessage::DiseaseText),
    ("--query", ErrorMessage::DiseaseText),
    ("--inheritance", ErrorMessage::DiseaseInheritance),
    ("--phenotype", ErrorMessage::DiseasePhenotypeFilter),
    ("--onset", ErrorMessage::DiseaseOnset),
    ("--no-fallback", ErrorMessage::DiseaseNoFallback),
    ("-l", ErrorMessage::DiseaseLimit),
    ("-j", ErrorMessage::DiseaseJson),
    ("--json", ErrorMessage::DiseaseJson),
    ("--no-cache", ErrorMessage::DiseaseFresh),
];
const SECTIONS: &[(&str, ErrorMessage)] = &[
    ("pathways", ErrorMessage::DiseasePathways),
    ("diagnostics", ErrorMessage::DiseaseDiagnostics),
    ("variants", ErrorMessage::DiseaseVariants),
    ("models", ErrorMessage::DiseaseModels),
    ("prevalence", ErrorMessage::DiseasePrevalence),
    ("funding", ErrorMessage::DiseaseFunding),
    ("civic", ErrorMessage::DiseaseCivic),
    ("disgenet", ErrorMessage::DiseaseDisgenet),
    ("all", ErrorMessage::DiseaseAll),
    ("clinical_features", ErrorMessage::DiseaseClinicalFeatures),
];
pub(super) fn message(root: &clap::Command, args: &[OsString]) -> Option<ErrorMessage> {
    let mut positional = Vec::new();
    let mut source = None;
    let mut section = None;
    let mut option = None;
    let mut shared_option = None;
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
        if matches!(flag, "-j" | "--json" | "--no-cache") {
            shared_option = super::find(FLAGS, flag);
        }
        let disease = matches!(positional.as_slice(), ["search" | "get", "disease", ..]);
        if disease && let Some(found) = super::find(FLAGS, flag) {
            return Some(found);
        }
        if disease && flag == "--name" {
            option = Some(ErrorMessage::DiseaseName);
        }
        if matches!(flag, "--name" | "--section")
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
    if shared_option.is_some() && matches!(positional.as_slice(), ["search" | "get", "disease", ..])
    {
        return shared_option;
    }
    match positional.as_slice() {
        ["list", "disease", ..] => Some(ErrorMessage::DiseaseHelp),
        ["disease", "drugs", ..] => Some(ErrorMessage::DiseaseDrugs),
        ["search", "disease", ..] if matches!(source, Some("mondo" | "doid" | "mesh")) => {
            Some(ErrorMessage::DiseaseOntologySelector)
        }
        ["get", "disease", id, rest @ ..] => {
            let section = section.or_else(|| (!literal).then(|| rest.first().copied()).flatten());
            if section == Some("genes") && source == Some("opentargets") {
                return Some(ErrorMessage::DiseaseOpenTargets);
            }
            if section == Some("survival") && source != Some("seer") {
                return Some(ErrorMessage::DiseaseSite);
            }
            if let Some(retired) = section.and_then(|s| super::find(SECTIONS, s)) {
                return Some(retired);
            }
            if id.parse::<biodata::Curie>().is_err() && source != Some("seer") {
                return Some(ErrorMessage::DiseaseName);
            }
            option
        }
        _ => option,
    }
}
