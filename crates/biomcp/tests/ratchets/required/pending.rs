//! Frozen hash admissions shrink together with the one editable pending list.
use super::Check;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub(super) fn read(
    text: &str,
    admitted: &str,
) -> Result<BTreeSet<String>, Box<dyn std::error::Error>> {
    let pending: BTreeSet<_> = text.lines().map(str::to_owned).collect();
    if pending.len() != text.lines().count() || pending.contains("") {
        return Err("required coverage: duplicate or empty pending identity".into());
    }
    let hashes: BTreeSet<_> = pending.iter().map(|item| hash(item)).collect();
    let admitted: BTreeSet<_> = admitted.lines().map(str::to_owned).collect();
    admissions(&hashes, &admitted)?;
    Ok(pending)
}
fn admissions(current: &BTreeSet<String>, admitted: &BTreeSet<String>) -> Check {
    if current != admitted {
        return Err("required coverage: pending identities and frozen admissions must shrink together; new pending identities are forbidden".into());
    }
    Ok(())
}
fn hash(item: &str) -> String {
    Sha256::digest(item.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
pub(super) fn admissions_retire_and_refuse_restoring_a_closed_gap() -> Check {
    let admitted = hash("option search trial --old-gap");
    read("option search trial --old-gap\n", &admitted)?;
    assert!(
        read("", &admitted).is_err(),
        "retired admission must leave with its pending identity"
    );
    read("", "")?;
    assert!(
        read("option search trial --old-gap\n", "").is_err(),
        "a closed gap cannot return"
    );
    assert!(
        read("option search trial --new-gap\n", &admitted).is_err(),
        "new gap cannot use an old admission"
    );
    Ok(())
}

// Generated once from the initial missing identities. Retire hashes with pending entries.
pub(super) const INITIAL_ADMISSIONS: &str = "\
";
