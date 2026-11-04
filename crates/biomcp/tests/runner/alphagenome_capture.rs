//! Feature variant-alphagenome: unchanged external scorer messages retain their native values.
use super::{SECTION, environment, fixture, invoke, routed};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(super) fn trials() -> Vec<Trial> {
    let Some(root) = std::env::var_os("BIOMCP_ALPHAGENOME_SCORE_CAPTURE") else {
        return Vec::new();
    };
    vec![transcript_trial(
        "variant-alphagenome/alphagenome-original-score-capture",
        move || captured(&PathBuf::from(&root)).map_err(Into::into),
    )]
}
fn checked(path: &Path, hash: &str) -> Result<Vec<u8>, String> {
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 1024 * 1024 {
        return Err("capture input exceeds bound".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
        != hash
    {
        return Err("external capture hash mismatch".into());
    }
    Ok(bytes)
}
fn captured(root: &Path) -> Result<(), String> {
    let manifest: Value = serde_json::from_slice(&checked(
        &root.join("raw/manifest.json"),
        "f1c353a6fd6b3d600bba85d2ae139c45a849bbdc2055c512b33df9925945724b",
    )?)
    .map_err(|e| e.to_string())?;
    let oracle: Vec<Value> = serde_json::from_slice(&checked(
        &root.join("oracle.json"),
        "5fcfde9d52c4192c142f64c2b50c86d8ee90c45848af20c23a0e9e5595820598",
    )?)
    .map_err(|e| e.to_string())?;
    let messages = manifest
        .as_object()
        .ok_or("capture manifest")?
        .iter()
        .filter(|(name, _)| name.ends_with(".bin"))
        .map(|(name, hash)| {
            checked(
                &root.join("raw").join(name),
                hash.as_str().ok_or("sidecar hash")?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if messages.len() != 6 {
        return Err("capture must retain six exact original messages".into());
    }
    let fixture = fixture(messages, 200)?;
    let env = environment(&fixture)?;
    let recording = env.root().join("original-replay");
    let seed = vec![
        "get".into(),
        "variant".into(),
        "CA123643".into(),
        "alphagenome".into(),
        "--record".into(),
        recording.to_string_lossy().into_owned(),
    ];
    invoke(&env, &seed, 0)?;
    // Keep the locally recorded resolver; replace its scorer exchange with the exact original.
    for (name, hash) in manifest.as_object().ok_or("capture manifest")? {
        if let Some(sidecar) = name.strip_prefix("recording/alphagenome/") {
            let bytes = checked(
                &root.join("raw").join(name),
                hash.as_str().ok_or("exchange hash")?,
            )?;
            std::fs::write(recording.join("alphagenome").join(sidecar), bytes)
                .map_err(|e| e.to_string())?;
        }
    }
    let offsets = [0, 4, 7, 304, 8671, 17341];
    for offset in offsets {
        let mut args = vec![
            "get".into(),
            "variant".into(),
            "CA123643".into(),
            "alphagenome".into(),
            "--section-offset".into(),
            offset.to_string(),
        ];
        args.extend(env.globals(Some(&recording), false));
        let reply = invoke(&env, &args, 0)?;
        let groups = reply
            .pointer(&format!("{SECTION}/data/scorers"))
            .and_then(Value::as_array)
            .ok_or("captured groups missing")?;
        if groups.len() != oracle.len() {
            return Err("captured scorer count changed".into());
        }
        for (group, expected) in groups.iter().zip(&oracle) {
            page(group, expected, offset)?;
        }
        if reply.pointer(&format!("{SECTION}/data/requested_model_version")) != Some(&json!(""))
            || reply
                .pointer(&format!("{SECTION}/data/model_version"))
                .is_some()
            || reply.pointer(&format!("{SECTION}/data/units")).is_some()
        {
            return Err("request defaults were misrepresented as native metadata".into());
        }
    }
    routed(&fixture, 1)?;
    fixture.finish().map_err(|e| e.to_string())
}
fn page(group: &Value, expected: &Value, offset: u32) -> Result<(), String> {
    if group.get("shape") != expected.get("shape")
        || group.get("data_type") != Some(&json!("DATA_TYPE_FLOAT32"))
    {
        return Err("captured native shape or dtype changed".into());
    }
    let shape = expected
        .get("shape")
        .and_then(Value::as_array)
        .ok_or("oracle shape")?;
    let tracks = shape
        .get(2)
        .and_then(Value::as_u64)
        .ok_or("oracle track dimension")?;
    let genes = shape
        .get(1)
        .and_then(Value::as_u64)
        .ok_or("oracle gene dimension")?;
    let values = expected
        .get("values")
        .and_then(Value::as_array)
        .ok_or("oracle values")?;
    let rows = group
        .get("rows")
        .and_then(Value::as_array)
        .ok_or("captured scalar rows")?;
    let count = values.len().saturating_sub(offset as usize).min(3);
    if rows.len() != count || group.pointer("/page/total") != Some(&json!(values.len())) {
        return Err("capture paging dropped native values".into());
    }
    for (returned, row) in rows.iter().enumerate() {
        let index = u64::from(offset) + returned as u64;
        let gene = index / tracks % genes;
        let track = index % tracks;
        if row.get("value") != values.get(index as usize)
            || row.get("tensor_index") != Some(&json!([index / tracks / genes, gene, track]))
            || row.pointer("/track/name") != expected.pointer(&format!("/tracks/{track}"))
            || row.pointer("/gene/gene_id") != expected.pointer(&format!("/genes/{gene}"))
        {
            return Err("captured scalar, coordinate or source metadata changed".into());
        }
    }
    Ok(())
}
