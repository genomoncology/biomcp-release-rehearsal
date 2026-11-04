//! Managed skill installation must preserve personal files and refuse path and symlink escape.
use super::launch::Environment;
use serde_json::{Value, json};
use std::{fs, path::Path};
type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;
pub(super) fn trial() -> libtest_mimic::Trial {
    super::transcript::trial("agent-skill/managed-files-and-symlink-boundaries", || {
        check().map_err(|e| e.to_string().into())
    })
}
fn run(env: &Environment, args: &[&str]) -> Check<Vec<u8>> {
    let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    let output = env.command(env!("CARGO_BIN_EXE_biomcp"), &args).output()?;
    if !output.status.success() {
        return Err(format!("skill command failed: {args:?}: {output:?}").into());
    }
    let measured = super::output_limit(if args.get(1).is_some_and(|word| word == "install") {
        "skill-install"
    } else {
        "skill-status"
    })?;
    if output.stdout.len() > (measured * 115).div_ceil(100) {
        return Err(format!("skill output exceeds measured ceiling: {args:?}").into());
    }
    Ok(output.stdout)
}
fn state(env: &Environment, directory: &str, expected: &str) -> Check {
    let bytes = run(env, &["skill", "status", directory])?;
    let value: Value = serde_json::from_slice(&bytes)?;
    if value.get("state") != Some(&json!(expected)) {
        return Err(format!("expected {expected}: {value}").into());
    }
    Ok(())
}
fn check() -> Check {
    let env = Environment::new()?;
    state(&env, "agent", "missing")?;
    run(&env, &["skill", "install", "agent"])?;
    state(&env, "agent", "current")?;
    let target = env.root().join("agent/skills/biomcp");
    let guide = fs::read(target.join("SKILL.md"))?;
    fs::write(target.join("SKILL.md"), "local change")?;
    state(&env, "agent", "locally_modified")?;
    let unchanged: Value = serde_json::from_slice(&run(&env, &["skill", "install", "agent"])?)?;
    if unchanged.get("changed") != Some(&json!(false)) {
        return Err("unforced install changed an existing tree".into());
    }
    fs::write(target.join("personal.txt"), "keep")?;
    run(&env, &["skill", "install", "agent", "--force"])?;
    if fs::read(target.join("SKILL.md"))? != guide
        || fs::read_to_string(target.join("personal.txt"))? != "keep"
    {
        return Err("repair lost guide or unrelated file".into());
    }
    let stamp = target.join(".biomcp-skill.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&stamp)?)?;
    *manifest
        .get_mut("biomcp_version")
        .ok_or("version missing")? = json!("0.9.3");
    fs::write(&stamp, serde_json::to_vec(&manifest)?)?;
    state(&env, "agent", "stale")?;
    fs::remove_file(&stamp)?;
    state(&env, "agent", "unmanaged")?;
    manifest_boundary(&env, &target)?;
    boundary(&env, &target)?;
    Ok(())
}
fn refused(env: &Environment, args: &[&str]) -> Check {
    let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    if env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()?
        .status
        .success()
    {
        return Err(format!("unsafe install accepted: {args:?}").into());
    }
    Ok(())
}
fn boundary(env: &Environment, target: &Path) -> Check {
    refused(env, &["skill", "install", "agent/../escape"])?;
    refused(env, &["skill", "install"])?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = env.root().join("outside");
        fs::create_dir(&outside)?;
        fs::write(outside.join("sentinel"), "unchanged")?;
        symlink(&outside, env.root().join("linked"))?;
        refused(env, &["skill", "install", "linked"])?;
        let path = target.join("SKILL.md");
        fs::remove_file(&path)?;
        symlink(outside.join("sentinel"), &path)?;
        refused(env, &["skill", "install", "agent", "--force"])?;
        refused(env, &["skill", "status", "agent"])?;
        if fs::read_to_string(outside.join("sentinel"))? != "unchanged"
            || fs::read_dir(&outside)?.count() != 1
        {
            return Err("install escaped the named directory".into());
        }
    }
    Ok(())
}
fn manifest_boundary(env: &Environment, target: &Path) -> Check {
    for path in [
        "../sentinel",
        "/sentinel",
        "use-cases/../../sentinel",
        "use-cases\\sentinel",
    ] {
        let stamp = json!({"schema_version":1,"biomcp_version":"1.0.0","managed_files":{path:"a".repeat(64)}});
        fs::write(
            target.join(".biomcp-skill.json"),
            serde_json::to_vec(&stamp)?,
        )?;
        refused(env, &["skill", "status", "agent"])?;
    }
    fs::remove_file(target.join(".biomcp-skill.json"))?;
    Ok(())
}
