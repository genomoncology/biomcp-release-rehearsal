//! Fixed migration guidance matched at command and section boundaries.

mod adverse_event;
mod author;
mod cell_line;
mod diagnostic;
mod drug;
mod pathway;
mod pgx;
mod phenotype;
mod protein;
mod study;

use biomcp_types::{ArticleSectionWord, ErrorMessage, GeneSectionWord, VariantSection};
use std::ffi::OsString;
mod disease;

const GENE: &[(&str, ErrorMessage)] = &[
    ("pathways", ErrorMessage::Pathways),
    ("trials", ErrorMessage::Trials),
    ("articles", ErrorMessage::Articles),
    ("drugs", ErrorMessage::Drugs),
    ("cspec", ErrorMessage::Variant),
];
const SEARCH: &[(&str, ErrorMessage)] = &[
    ("--type", ErrorMessage::LocusGroup),
    ("--chromosome", ErrorMessage::Band),
    ("--region", ErrorMessage::Dropped),
    ("--pathway", ErrorMessage::Dropped),
    ("--go", ErrorMessage::Dropped),
];
const SECTIONS: &[(&str, ErrorMessage)] = &[
    ("hpa", ErrorMessage::Expression),
    ("ontology", ErrorMessage::Dropped),
    ("diseases", ErrorMessage::Dropped),
    ("go", ErrorMessage::Dropped),
    ("interactions", ErrorMessage::Dropped),
    ("funding", ErrorMessage::Dropped),
    ("disgenet", ErrorMessage::Dropped),
    ("druggability", ErrorMessage::Drug),
    ("diagnostics", ErrorMessage::Diagnostic),
];
const ARTICLE_SEARCH: &[(&str, ErrorMessage)] = &[
    ("-k", ErrorMessage::ArticleText),
    ("--date-from", ErrorMessage::Since),
    ("--year-min", ErrorMessage::Since),
    ("--date-to", ErrorMessage::Until),
    ("--year-max", ErrorMessage::Until),
    ("--weight-citations", ErrorMessage::CitationSort),
    ("-g", ErrorMessage::GeneOption),
    ("-d", ErrorMessage::DiseaseOption),
    ("-a", ErrorMessage::AuthorOption),
    ("--ranking-mode", ErrorMessage::ScoreBlends),
    ("--weight-semantic", ErrorMessage::ScoreBlends),
    ("--weight-lexical", ErrorMessage::ScoreBlends),
    ("--weight-position", ErrorMessage::ScoreBlends),
    ("--max-per-source", ErrorMessage::Dropped),
    ("--session", ErrorMessage::Dropped),
    ("--debug-plan", ErrorMessage::Dropped),
    ("--full", ErrorMessage::Dropped),
];
const TRIAL_SEARCH: &[(&str, ErrorMessage)] = &[
    ("--line-of-therapy", ErrorMessage::TrialCriteria),
    ("--prior-therapies", ErrorMessage::TrialCriteria),
    ("--progression-on", ErrorMessage::TrialCriteria),
    ("--mutation", ErrorMessage::TrialVariant),
    ("--biomarker", ErrorMessage::TrialVariant),
    ("--date-from", ErrorMessage::TrialSince),
    ("--date-to", ErrorMessage::TrialUntil),
    ("--results-available", ErrorMessage::TrialResults),
    ("--no-alias-expand", ErrorMessage::TrialNoExpansion),
];
const GWAS_SEARCH: &[(&str, ErrorMessage)] = &[];
const ARTICLE_GET: &[(&str, ErrorMessage)] = &[
    ("--pdf", ErrorMessage::ArticlePdf),
    ("--outline", ErrorMessage::Fulltext),
    ("--lines", ErrorMessage::ArticlePart),
    ("--out", ErrorMessage::Dropped),
    ("--output", ErrorMessage::Dropped),
    ("--asset-view", ErrorMessage::Dropped),
    ("--asset-limit", ErrorMessage::Dropped),
    ("--asset-offset", ErrorMessage::Dropped),
];
const ARTICLE_SECTIONS: &[(&str, ErrorMessage)] = &[
    ("assets", ErrorMessage::Supplements),
    ("asset", ErrorMessage::Supplement),
];
const VARIANT_SECTIONS: &[(&str, ErrorMessage)] = &[
    ("predictions", ErrorMessage::VariantPredict),
    ("population-details", ErrorMessage::VariantPopulation),
    ("cbioportal", ErrorMessage::VariantRecurrence),
    ("cgi", ErrorMessage::VariantCgi),
];
const VARIANT_SEARCH: &[(&str, ErrorMessage)] = &[("--hgvsp", ErrorMessage::VariantProtein)];
const VARIANT_GET: &[(&str, ErrorMessage)] = &[("--assembly", ErrorMessage::VariantBuild)];
const VARIANT_NORMALIZE: &[(&str, ErrorMessage)] = &[("--input", ErrorMessage::VariantNormalize)];
fn find(table: &[(&str, ErrorMessage)], word: &str) -> Option<ErrorMessage> {
    table
        .iter()
        .find_map(|(key, value)| (*key == word).then_some(*value))
}

pub(crate) fn message(root: &clap::Command, args: &[OsString]) -> Option<ErrorMessage> {
    if let Some(message) = author::message(root, args) {
        return Some(message);
    }
    if retired_source(root, args) {
        return Some(ErrorMessage::Pharmgkb);
    }
    if let Some(message) = study::message(root, args) {
        return Some(message);
    }
    if let Some(message) = pathway::message(root, args) {
        return Some(message);
    }
    if let Some(message) = adverse_event::message(root, args) {
        return Some(message);
    }
    if let Some(message) = phenotype::message(root, args) {
        return Some(message);
    }
    if let Some(message) = disease::message(root, args) {
        return Some(message);
    }
    if let Some(message) = pgx::message(root, args) {
        return Some(message);
    }
    if let Some(message) = diagnostic::message(root, args) {
        return Some(message);
    }
    let words = words(root, args)?;
    if let Some(message) = cell_line::message(&words) {
        return Some(message);
    }
    if let Some(message) = protein::message(&words) {
        return Some(message);
    }
    if let Some(message) = command(&words.positional) {
        return Some(message);
    }
    match words.positional.as_slice() {
        ["gene", rest @ ..] if !rest.is_empty() => {
            find(GENE, rest.first()?).or(Some(ErrorMessage::Gene))
        }
        ["search", "drug", ..] | ["search", "gwas", ..] => words.retired_option,
        ["get", "drug", input @ ..] => words
            .retired_option
            .or_else(|| drug::section(&words, input)),
        ["search", "gene", ..] | ["search", "trial", ..] => words.retired_option,
        ["get", "gene", input @ ..] => trailing(&words, input, SECTIONS, false),
        ["search", "article", ..] | ["search", "variant", ..] | ["variant", "normalize", ..] => {
            words.retired_option
        }
        ["get", "variant", input @ ..] => words
            .retired_option
            .or_else(|| variant_section(&words, input)),
        ["get", "article", input @ ..] => words
            .retired_option
            .or_else(|| trailing(&words, input, ARTICLE_SECTIONS, true)),
        _ => None,
    }
}
fn command(positional: &[&str]) -> Option<ErrorMessage> {
    if let Some(message) = cell_line::command(positional) {
        return Some(message);
    }
    if let Some(message) = drug::command(positional) {
        return Some(message);
    }
    match positional {
        ["variant", "structure", ..] => Some(ErrorMessage::VariantStructure),
        ["variant", "oncokb", ..] => Some(ErrorMessage::VariantOncokb),
        ["variant", "erepo", ..] => Some(ErrorMessage::VariantErepo),
        ["variant", "cspec", ..] => Some(ErrorMessage::VariantCspec),
        [
            "variant",
            "normalize",
            "car" | "mutalyzer" | "variantvalidator" | "all",
            ..,
        ] => Some(ErrorMessage::VariantNormalize),
        ["gene", word] => find(GENE, word),
        ["batch", "trial", ..] => Some(ErrorMessage::TrialBatch),
        ["variant", "trials", ..] => Some(ErrorMessage::VariantTrials),
        ["disease", "trials", ..] => Some(ErrorMessage::DiseaseTrials),
        ["drug", "trials", ..] => Some(ErrorMessage::DrugTrials),
        ["get", "trial", _, "document", ..] => Some(ErrorMessage::TrialDocument),
        ["article", "supplement-search", ..] => Some(ErrorMessage::SupplementFind),
        ["article", "entities", ..] => Some(ErrorMessage::Annotations),
        ["article", "batch" | "authors", ..] | ["batch", "article", ..] => {
            Some(ErrorMessage::ArticleBatch)
        }
        ["variant", "articles", ..] => Some(ErrorMessage::VariantArticles),
        ["disease", "articles", ..] => Some(ErrorMessage::DiseaseArticles),
        ["drug", "articles", ..] => Some(ErrorMessage::DrugArticles),
        ["pathway", "articles", ..] => Some(ErrorMessage::Pathway),
        _ => None,
    }
}
fn trailing(
    words: &Words<'_>,
    input: &[&str],
    table: &[(&str, ErrorMessage)],
    article: bool,
) -> Option<ErrorMessage> {
    if !words.sections.is_empty() {
        return words.sections.iter().find_map(|word| find(table, word));
    }
    if words.literal {
        return None;
    }
    input
        .get(1..)
        .unwrap_or_default()
        .iter()
        .rev()
        .take_while(|word| {
            find(table, word).is_some()
                || if article {
                    serde_json::from_value::<ArticleSectionWord>(serde_json::Value::String(
                        (**word).into(),
                    ))
                    .is_ok()
                } else {
                    serde_json::from_value::<GeneSectionWord>(serde_json::Value::String(
                        (**word).into(),
                    ))
                    .is_ok()
                }
        })
        .find_map(|word| find(table, word))
}
fn variant_section(words: &Words<'_>, input: &[&str]) -> Option<ErrorMessage> {
    if !words.sections.is_empty() {
        return words
            .sections
            .iter()
            .find_map(|word| find(VARIANT_SECTIONS, word));
    }
    if words.literal {
        return None;
    }
    input
        .get(1..)
        .unwrap_or_default()
        .iter()
        .rev()
        .take_while(|word| {
            find(VARIANT_SECTIONS, word).is_some()
                || serde_json::from_value::<VariantSection>(serde_json::Value::String(
                    (**word).into(),
                ))
                .is_ok()
        })
        .find_map(|word| find(VARIANT_SECTIONS, word))
}
struct Words<'a> {
    positional: Vec<&'a str>,
    literal: bool,
    sections: Vec<&'a str>,
    retired_option: Option<ErrorMessage>,
}
fn words<'a>(root: &clap::Command, args: &'a [OsString]) -> Option<Words<'a>> {
    let mut result = Words {
        positional: Vec::new(),
        literal: false,
        sections: Vec::new(),
        retired_option: None,
    };
    let mut args = args.iter();
    let mut literal = false;
    while let Some(arg) = args.next() {
        let arg = arg.to_str()?;
        if literal {
            result.positional.push(arg);
            continue;
        }
        if arg == "--" {
            if result.positional.len() < 2 {
                return None;
            }
            literal = true;
            result.literal = true;
            continue;
        }
        if !arg.starts_with('-') {
            result.positional.push(arg);
            if command(&result.positional).is_some() {
                return Some(result);
            }
            continue;
        }
        let (flag, inline) = arg
            .split_once('=')
            .map_or((arg, None), |(key, value)| (key, Some(value)));
        let table = match result.positional.get(..2) {
            Some(["search", "drug"]) => drug::SEARCH,
            Some(["search", "protein"]) => protein::SEARCH,
            Some(["search", "gwas"]) => GWAS_SEARCH,
            Some(["get", "drug"]) => drug::GET,
            Some(["search", "article"]) => ARTICLE_SEARCH,
            Some(["search", "trial"]) => TRIAL_SEARCH,
            Some(["get", "article"]) => ARTICLE_GET,
            Some(["search", "gene"]) => SEARCH,
            Some(["get", "variant"]) => VARIANT_GET,
            Some(["search", "variant"]) => VARIANT_SEARCH,
            Some(["variant", "normalize"]) => VARIANT_NORMALIZE,
            _ => &[],
        };
        let retired = find(table, flag);
        if retired.is_some() {
            result.retired_option = result.retired_option.or(retired);
        } else if !takes_value(root, flag, &result.positional)? && inline.is_none() {
            continue;
        }
        let value = inline.or_else(|| args.next().and_then(|arg| arg.to_str()));
        if retired == Some(ErrorMessage::DrugRegion)
            && value == Some("who")
            && result.retired_option == retired
        {
            result.retired_option = Some(ErrorMessage::DrugWhoRegion);
        }
        if flag == "--section" {
            result.sections.push(value?);
        }
        if value.is_none() && retired.is_none() {
            return None;
        }
    }
    Some(result)
}

fn takes_value(root: &clap::Command, flag: &str, positional: &[&str]) -> Option<bool> {
    if flag == "--section" && matches!(positional.get(..2), Some(["get", "cell-line" | "drug"])) {
        return Some(true);
    }
    if flag == "-l" && matches!(positional.get(..2), Some(["search", "phenotype"])) {
        return Some(true);
    }
    if let Some(argument) = root
        .get_arguments()
        .find(|argument| argument.get_long() == flag.strip_prefix("--"))
    {
        return Some(argument.get_action().takes_values());
    }
    let Some([verb, entity]) = positional.get(..2) else {
        return None;
    };
    let (_, _, id) = crate::grammar::command_paths()
        .into_iter()
        .find(|row| row.0 == *verb && row.1 == *entity)?;
    crate::schema::find(id)
        .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
        .and_then(|schema| {
            schema
                .get("properties")
                .and_then(serde_json::Value::as_object)
                .and_then(|properties| {
                    properties
                        .iter()
                        .find(|(name, property)| {
                            property.get("x-positional").is_none()
                                && flag == format!("--{}", name.replace('_', "-"))
                        })
                        .map(|(_, property)| {
                            property.get("type").and_then(serde_json::Value::as_str)
                                != Some("boolean")
                        })
                })
        })
}

fn retired_source(root: &clap::Command, args: &[OsString]) -> bool {
    let mut positional = Vec::new();
    let mut args = args.iter();
    while let Some(word) = args.next().and_then(|word| word.to_str()) {
        if word == "--" {
            break;
        }
        if !word.starts_with('-') {
            positional.push(word);
            continue;
        }
        let (flag, value) = word
            .split_once('=')
            .map_or((word, None), |(flag, value)| (flag, Some(value)));
        if matches!(flag, "--source" | "--disable-sources") {
            let value = value.or_else(|| args.next().and_then(|word| word.to_str()));
            if value.is_some_and(|value| value.split(',').any(|part| part == "pharmgkb")) {
                return true;
            }
        } else if value.is_none() && takes_value(root, flag, &positional).unwrap_or(false) {
            args.next();
        }
    }
    false
}
