//! Literal protocol context for registry gene-word matches; never eligibility rules.
use biodata::NctId;
use std::ops::Range;

/// One retained trial with a factual gene-word caution.
#[derive(Debug)]
pub struct GeneContext {
    /// Stable registry row identity.
    pub nct_id: NctId,
    /// Where the literal gene text was observed.
    pub mention: GeneMention,
}

/// Observed source wording, without assigning mutation or enrollment status.
#[derive(Debug)]
pub enum GeneMention {
    /// All observed whole-word mentions occur below an explicit exclusion heading.
    ExclusionOnly {
        /// Bounded original exclusion text.
        context: String,
    },
    /// A source field explicitly uses negative or wild-type gene wording.
    NegativeContext {
        /// The returned field containing the wording.
        field: &'static str,
        /// Original short phrase, preserving source case.
        context: String,
    },
    /// No whole-word mention occurs in the returned protocol fields.
    NotMentioned,
}

pub(super) fn classify(
    gene: &str,
    fields: &[(&'static str, &str)],
    eligibility: Option<&str>,
) -> Option<GeneMention> {
    if gene.is_empty() {
        return None;
    }
    let mut outside = false;
    let mut exclusion = None;
    let mut negative = None;
    for &(field, text) in fields {
        let matches = mentions(text, gene);
        outside |= !matches.is_empty();
        for range in matches {
            if negative.is_none() {
                negative = negative_phrase(text, range).map(|context| (field, context));
            }
        }
    }
    let mut excluded = false;
    for line in eligibility.unwrap_or_default().lines() {
        if let Some(section) = exclusion_heading(line) {
            excluded = section;
        }
        for range in mentions(line, gene) {
            if excluded {
                if exclusion.is_none() {
                    exclusion = excerpt(line, range.clone());
                }
            } else {
                outside = true;
            }
            if negative.is_none() {
                negative =
                    negative_phrase(line, range).map(|context| ("eligibility text", context));
            }
        }
    }
    if !outside && let Some(context) = exclusion {
        Some(GeneMention::ExclusionOnly { context })
    } else if let Some((field, context)) = negative {
        Some(GeneMention::NegativeContext { field, context })
    } else if !outside {
        Some(GeneMention::NotMentioned)
    } else {
        None
    }
}

fn exclusion_heading(line: &str) -> Option<bool> {
    let heading = line.trim_matches(|c: char| c.is_whitespace() || "#*_-".contains(c));
    let heading = heading.to_ascii_lowercase();
    let heading = heading.strip_prefix("key ").unwrap_or(&heading);
    [("exclusion criteria", true), ("inclusion criteria", false)]
        .into_iter()
        .find_map(|(prefix, excluded)| {
            heading
                .strip_prefix(prefix)
                .filter(|tail| tail.is_empty() || tail.starts_with(':'))
                .map(|_| excluded)
        })
}

fn mentions(text: &str, gene: &str) -> Vec<Range<usize>> {
    let upper = text.to_ascii_uppercase();
    let symbol = gene.to_ascii_uppercase();
    upper
        .match_indices(symbol.as_str())
        .filter_map(|(start, found)| {
            let end = start + found.len();
            let word = |c: char| c.is_alphanumeric() || c == '_';
            (!text.get(..start)?.chars().next_back().is_some_and(word)
                && !text.get(end..)?.chars().next().is_some_and(word))
            .then_some(start..end)
        })
        .collect()
}

fn negative_phrase(text: &str, gene: Range<usize>) -> Option<String> {
    let after = text.get(gene.end..)?;
    let suffix = after.trim_start_matches(|c: char| c.is_whitespace() || c == '-');
    let leading = after.len() - suffix.len();
    for marker in ["wild type", "wild-type", "wildtype", "wt", "negative"] {
        if suffix
            .get(..marker.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(marker))
            && !suffix
                .get(marker.len()..)
                .and_then(|tail| tail.chars().next())
                .is_some_and(char::is_alphanumeric)
        {
            return text
                .get(gene.start..gene.end + leading + marker.len())
                .map(phrase);
        }
    }
    let before = text.get(..gene.start)?.trim_end();
    for marker in [
        "without",
        "absence of any",
        "absence of",
        "wild type",
        "wild-type",
    ] {
        if before.len() >= marker.len() {
            let start = before.len() - marker.len();
            if before
                .get(start..)
                .is_some_and(|s| s.eq_ignore_ascii_case(marker))
                && !before
                    .get(..start)
                    .and_then(|prefix| prefix.chars().next_back())
                    .is_some_and(char::is_alphanumeric)
            {
                let tail = suffix
                    .split(|c: char| !c.is_alphanumeric())
                    .next()
                    .unwrap_or_default();
                let end = if tail.eq_ignore_ascii_case("mutation")
                    || tail.eq_ignore_ascii_case("mutations")
                {
                    gene.end + leading + tail.len()
                } else {
                    gene.end
                };
                return text.get(start..end).map(phrase);
            }
        }
    }
    None
}

fn excerpt(text: &str, gene: Range<usize>) -> Option<String> {
    let before: String = text.get(..gene.start)?.chars().rev().take(60).collect();
    let before: String = before.chars().rev().collect();
    let after: String = text.get(gene.end..)?.chars().take(60).collect();
    Some(phrase(&format!("{before}{}{after}", text.get(gene)?)))
}

fn phrase(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
