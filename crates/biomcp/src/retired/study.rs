//! Study migration guidance respects argument boundaries and never reads local data.
use biomcp_types::ErrorMessage;
use std::ffi::OsString;
pub(super) fn message(root: &clap::Command, args: &[OsString]) -> Option<ErrorMessage> {
    let mut positional = Vec::new();
    let mut chart = false;
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
            .map_or((word, None), |(key, value)| (key, Some(value)));
        chart |= CHARTS.contains(&flag);
        if inline.is_none()
            && (VALUE_FLAGS.contains(&flag)
                || super::takes_value(root, flag, &positional).unwrap_or(false))
        {
            args.next();
        }
    }
    let study = matches!(
        positional.as_slice(),
        ["study", ..]
            | ["search" | "get" | "list", "study", ..]
            | ["sync", "cbioportal-studies", ..]
    );
    if !study {
        return None;
    }
    if chart {
        return Some(ErrorMessage::StudyCharts);
    }
    match positional.as_slice() {
        ["study", "list", ..] | ["list", "study", ..] => Some(ErrorMessage::StudyList),
        ["study", "download", ..] => Some(ErrorMessage::StudyDownload),
        [
            "study",
            "query" | "top-mutated" | "filter" | "cohort" | "survival" | "compare"
            | "co-occurrence",
            ..,
        ] => Some(ErrorMessage::StudyCalculate),
        _ if crate::config::retired_study_directory() => Some(ErrorMessage::StudyDirectory),
        _ => None,
    }
}
const CHARTS: &[&str] = &[
    "--chart",
    "--terminal",
    "--output",
    "-o",
    "--title",
    "--theme",
    "--palette",
    "--cols",
    "--rows",
    "--width",
    "--height",
    "--scale",
    "--mcp-inline",
];
const VALUE_FLAGS: &[&str] = &[
    "--study",
    "-s",
    "--gene",
    "--genes",
    "-g",
    "--type",
    "-t",
    "--limit",
    "-l",
    "--endpoint",
    "-e",
    "--target",
    "--mutated",
    "--amplified",
    "--deleted",
    "--expression-above",
    "--expression-below",
    "--cancer-type",
    "--output",
    "-o",
    "--title",
    "--theme",
    "--palette",
    "--cols",
    "--rows",
    "--width",
    "--height",
    "--scale",
];
