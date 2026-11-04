//! Fixed pathway retirements never rewrite input or dispatch a provider request.
use biomcp_types::ErrorMessage;
use std::ffi::OsString;
pub(super) fn message(root: &clap::Command, args: &[OsString]) -> Option<ErrorMessage> {
    let mut positional = Vec::new();
    let mut args = args.iter();
    while let Some(word) = args.next() {
        let word = word.to_str()?;
        if word == "--" {
            break;
        }
        if !word.starts_with('-') {
            positional.push(word);
            if let Some(message) = command(&positional) {
                return Some(message);
            }
            continue;
        }
        let (flag, inline) = word
            .split_once('=')
            .map_or((word, None), |(a, b)| (a, Some(b)));
        if matches!(positional.get(..2), Some(["search", "pathway"])) {
            if flag == "-l" {
                if inline.is_none() {
                    args.next();
                }
                continue;
            }
            if flag
                .strip_prefix("-l")
                .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
            {
                continue;
            }
            match flag {
                "-q" | "--query" => return Some(ErrorMessage::PathwayText),
                "--type" => return Some(ErrorMessage::PathwayClass),
                _ => (),
            }
        }
        if matches!(positional.get(..2), Some(["get", "pathway"])) && flag == "--section" {
            let section = inline.or_else(|| args.next().and_then(|v| v.to_str()));
            if section == Some("enrichment") {
                return Some(ErrorMessage::PathwayEnrich);
            }
        } else if inline.is_none() && super::takes_value(root, flag, &positional)? {
            args.next();
        }
    }
    None
}
fn command(words: &[&str]) -> Option<ErrorMessage> {
    Some(match words {
        ["enrich", ..] => ErrorMessage::PathwayEnrich,
        ["list", "pathway", ..] => ErrorMessage::PathwayHelp,
        ["pathway", "drugs", ..] => ErrorMessage::PathwayDrugs,
        ["pathway", "articles", ..] => ErrorMessage::PathwayArticles,
        ["pathway", "trials", ..] => ErrorMessage::PathwayTrials,
        ["get", "pathway", id, ..] if kegg(id) => ErrorMessage::PathwayKegg,
        ["get", "pathway", _, "enrichment", ..] => ErrorMessage::PathwayEnrich,
        _ => return None,
    })
}
fn kegg(id: &str) -> bool {
    let id = id.strip_prefix("path:").unwrap_or(id);
    let letters = id.bytes().take_while(u8::is_ascii_lowercase).count();
    matches!(letters, 2 | 3)
        && id
            .get(letters..)
            .is_some_and(|digits| digits.len() == 5 && digits.bytes().all(|b| b.is_ascii_digit()))
}
