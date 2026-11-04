//! Author migration guidance preserves literal queries and exact source identities.
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
            match positional.as_slice() {
                ["list", "author", ..] => return Some(ErrorMessage::AuthorHelp),
                ["get", "author", _, "coauthors", ..] => return Some(ErrorMessage::AuthorDeferred),
                ["author", "rank" | "ranking" | "coauthors" | "merge", ..] => {
                    return Some(ErrorMessage::AuthorDeferred);
                }
                _ => (),
            }
            continue;
        }
        let (flag, inline) = word
            .split_once('=')
            .map_or((word, None), |(k, v)| (k, Some(v)));
        if matches!(positional.get(..2), Some(["search", "author"])) {
            if matches!(
                flag,
                "--topic" | "--rank" | "--ranking" | "--coauthors" | "--merge"
            ) {
                return Some(ErrorMessage::AuthorDeferred);
            }
            if flag == "--source" {
                let value = inline.or_else(|| args.next().and_then(|v| v.to_str()));
                if value == Some("orcid") {
                    return Some(ErrorMessage::AuthorOrcidSearch);
                }
                continue;
            }
            if flag
                .strip_prefix("-l")
                .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
            {
                continue;
            }
            // The short limit flag retains its ordinary value boundary.
            if flag == "-l" {
                if inline.is_none() {
                    args.next();
                }
                continue;
            }
        }
        if inline.is_none() && super::takes_value(root, flag, &positional)? {
            args.next();
        }
    }
    None
}
