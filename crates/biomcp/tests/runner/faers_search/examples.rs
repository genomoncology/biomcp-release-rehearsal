//! Generate synthetic local recordings through the real transport recorder.
use super::{Check, body, environment};
use std::path::Path;
pub(super) fn record(directory: &Path) -> Check {
    std::fs::create_dir_all(directory.join("faers")).map_err(|e| e.to_string())?;
    let stub = super::Stub::for_sources(
        &["api.fda.gov"],
        |_| "application/json",
        |request| {
            if !request.starts_with("GET /drug/event.json?search=")
                || !request.contains("&limit=5&skip=0&sort=receivedate%3Adesc ")
            {
                return Err("unexpected synthetic example request".into());
            }
            if request.contains("search=safetyreportid%3A%22100%22&") {
                let mut report = body(1, 0);
                *report.pointer_mut("/meta/results/total").ok_or("total")? = serde_json::json!(1);
                *report.pointer_mut("/meta/results/limit").ok_or("limit")? = serde_json::json!(5);
                Ok(report.to_string())
            } else {
                Ok(body(5, 0).to_string())
            }
        },
    )?;
    for (index, words) in commands().into_iter().enumerate() {
        let env = environment(&stub)?;
        let words = words.into_iter().map(str::to_owned);
        let recording = env.root().join(format!("record-{index}"));
        let mut args = vec!["--record".into(), recording.to_string_lossy().into()];
        args.extend(words);
        super::invoke(&env, args, 0)?;
        for entry in std::fs::read_dir(recording.join("faers")).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().is_some_and(|s| s == "json") {
                std::fs::copy(
                    entry.path(),
                    directory.join("faers").join(entry.file_name()),
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    if std::fs::read_dir(directory.join("faers"))
        .map_err(|e| e.to_string())?
        .count()
        != 7
    {
        return Err("synthetic recordings were not retained".into());
    }
    if stub.finish()?.len() != 8 {
        return Err("synthetic examples did not record eight actual requests".into());
    }
    Ok(())
}

fn commands() -> [Vec<&'static str>; 8] {
    [
        vec!["search", "adverse-event", "pembrolizumab"],
        vec![
            "search",
            "adverse-event",
            "pembrolizumab",
            "--received-from",
            "2024-01-01",
            "--received-to",
            "2024-12-31",
        ],
        vec![
            "search",
            "adverse-event",
            "pembrolizumab",
            "--serious",
            "lifethreatening",
        ],
        vec![
            "search",
            "adverse-event",
            "pembrolizumab",
            "--serious",
            "disabling",
        ],
        vec![
            "search",
            "adverse-event",
            "pembrolizumab",
            "--serious",
            "congenital-anomaly",
        ],
        vec![
            "search",
            "adverse-event",
            "pembrolizumab",
            "--serious",
            "other",
        ],
        vec!["get", "adverse-event", "100"],
        vec!["get", "adverse-event", "100", "reactions"],
    ]
}
