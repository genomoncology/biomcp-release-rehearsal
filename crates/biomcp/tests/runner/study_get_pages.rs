//! Explicit bounded metadata pages preserve authored synthetic source order.
use super::{Check, call};
use crate::runner::launch::Environment;
use serde_json::json;
use std::path::Path;
pub(super) fn limited(env: &Environment, replay: &Path) -> Check {
    let limited = call(
        env,
        replay,
        &[
            "get",
            "study",
            "synthetic_0",
            "profiles",
            "--section-limit",
            "2",
            "--section-offset",
            "1",
        ],
        0,
        "get/pages/limited",
    )?;
    if limited.pointer("/page/returned") != Some(&json!(2))
        || limited.pointer("/result/items/0/molecularProfileId") != Some(&json!("native_profile_1"))
    {
        return Err("explicit section page lost its bound or source order".into());
    }
    Ok(())
}
