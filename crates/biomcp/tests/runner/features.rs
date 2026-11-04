//! Feature ownership is checked before any case launches the product.
use std::collections::BTreeSet;

pub(super) fn validate<'a>(
    text: &str,
    trials: impl IntoIterator<Item = &'a str>,
    complete: bool,
) -> Result<(), String> {
    let mut listed = BTreeSet::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (name, sentence) = line
            .split_once(": ")
            .ok_or("feature requires name: sentence")?;
        if !name.starts_with(|c: char| c.is_ascii_lowercase())
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || !sentence.ends_with(['.', '!', '?'])
            || sentence.contains(" Cases:")
            || !listed.insert(name)
        {
            return Err(format!("invalid or duplicate feature: {line}"));
        }
    }
    let mut covered = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut errors = Vec::new();
    for trial in trials {
        if !names.insert(trial) {
            errors.push(format!("duplicate trial: {trial}"));
        }
        let Some((feature, _)) = trial.split_once('/') else {
            errors.push(format!("trial lacks feature: {trial}"));
            continue;
        };
        if !listed.contains(feature) {
            errors.push(format!("unlisted feature {feature}: {trial}"));
        }
        covered.insert(feature);
    }
    let missing = listed.difference(&covered).copied().collect::<Vec<_>>();
    if complete && !missing.is_empty() {
        errors.push(format!("features without tests: {}", missing.join(", ")));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}
