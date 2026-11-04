//! Retired phenotype forms never rewrite text or choose a candidate.
use biomcp_types::ErrorMessage;
use std::ffi::OsString;
pub(super) fn message(root: &clap::Command, args: &[OsString]) -> Option<ErrorMessage> {
    let words = super::words(root, args)?;
    match words.positional.as_slice() {
        ["list", "phenotype", ..] => Some(ErrorMessage::PhenotypeHelp),
        ["search", "phenotype", text @ ..] => {
            let mut ids = text
                .iter()
                .flat_map(|value| value.split(|c: char| c == ',' || c.is_whitespace()))
                .filter(|v| !v.is_empty())
                .peekable();
            if ids.peek().is_some() && ids.all(|id| id.parse::<biodata::HpId>().is_ok()) {
                return Some(ErrorMessage::PhenotypeSet);
            }
            let mut args = args.iter().filter_map(|v| v.to_str());
            while let Some(word) = args.next() {
                if word == "--" {
                    break;
                }
                if word == "--source=hpo" || (word == "--source" && args.next() == Some("hpo")) {
                    return Some(ErrorMessage::PhenotypeHpoSearch);
                }
            }
            None
        }
        _ => None,
    }
}
