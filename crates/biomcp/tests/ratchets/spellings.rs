//! Audit BioMCP wire values while preserving BioData and publisher spellings.
use super::{Check, rust_files};
use std::{fs, path::Path};

// These declarations describe BioMCP choices rather than publisher vocabularies.
const OWNED: &[&str] = &[
    "Source",
    "Use",
    "SourceState",
    "ProblemCode",
    "ArticleSource",
    "PgxSource",
    "PgxSection",
    "PgxAnnotationKind",
    "PgxTotalKind",
    "Reason",
    "PublicationType",
    "SubjectQualification",
    "DrugOncokbMissing",
    "AdverseEventReactionMatch",
    "ArticleMatchKind",
    "CorrectionKind",
    "DiagnosticCountUnit",
    "SourceTimestampTimezone",
    "OrcidWorksScope",
    "BundleSize",
    "ConfigurationNoticeCode",
    "RecurrenceGranularity",
    "AlphaGenomeNonfiniteKind",
    "EvidenceNotice",
    "PathwayTotalKind",
    "AlphaGenomeScorer",
    "DrugRecallCountUnit",
];

pub(super) fn check(root: &Path) -> Check {
    for name in ["biodata", "biomcp-types"] {
        for path in rust_files(&root.join("crates").join(name).join("src"))? {
            let text = fs::read_to_string(&path)?;
            audit(&super::source::tokens(&text), name == "biomcp-types")
                .map_err(|error| format!("{}: {error}", path.display()))?;
        }
    }
    Ok(())
}

pub(super) fn audit(tokens: &[String], biomcp_owned: bool) -> Result<(), String> {
    for window in tokens.windows(3) {
        let [name, equal, value] = window else {
            continue;
        };
        if biomcp_owned && name == "rename_all" && value == "\"snake_case\"" {
            return Err("owned enum values require kebab-case".into());
        }
        if name == "tag" && equal == "=" && value.starts_with('"') {
            let tag = value.trim_matches('"');
            if tag == "annotationkind"
                || !tag
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
            {
                return Err(
                    "serde tags require snake_case names, including annotation_kind".into(),
                );
            }
        }
    }
    if !biomcp_owned {
        return Ok(());
    }
    for (index, token) in tokens.iter().enumerate() {
        // Error identifiers keep the approved dotted snake_case convention.
        if matches!(token.as_str(), "ErrorCode" | "ProblemCode") {
            continue;
        }
        let macro_position = tokens
            .get(index.wrapping_sub(1))
            .is_some_and(|previous| ["]", "{", "described"].contains(&previous.as_str()));
        if !macro_position || !OWNED.contains(&token.as_str()) {
            continue;
        }
        let tail = tokens.get(index + 1..).unwrap_or_default();
        let declaration = matches!(tail, [brace, ..] if brace == "{")
            || matches!(tail, [comma, _, brace, ..] if comma == "," && brace == "{");
        if !declaration {
            continue;
        }
        let Some(start) = tail.iter().position(|token| token == "{") else {
            continue;
        };
        for value in tail
            .get(start..)
            .unwrap_or_default()
            .windows(3)
            .take_while(|w| w.first().is_none_or(|token| token != "}"))
        {
            let [equal, arrow, literal] = value else {
                continue;
            };
            if equal == "=" && arrow == ">" {
                let literal = literal.trim_matches('"');
                if !literal
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
                {
                    return Err(format!("{token} requires kebab-case: {literal}"));
                }
            }
        }
    }
    Ok(())
}
