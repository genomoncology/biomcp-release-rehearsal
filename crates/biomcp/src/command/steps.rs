//! Spell checked follow-ups and retain the caller's global restrictions.
use std::io;
pub(super) fn next(
    result: &biomcp_query::Outcome,
    globals: &crate::grammar::Globals,
) -> io::Result<Vec<biomcp_types::Step>> {
    let steps = result
        .next
        .iter()
        .map(|step| crate::spell::step(step.0, &step.1))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(source_steps(globals, steps))
}

fn source_steps(
    globals: &crate::grammar::Globals,
    steps: Vec<biomcp_types::Step>,
) -> Vec<biomcp_types::Step> {
    let mut prefix: Vec<Box<str>> = Vec::new();
    if let Some(path) = &globals.config {
        prefix.extend([
            "--config".into(),
            path.to_string_lossy().into_owned().into(),
        ]);
    }
    if !globals.disable_sources.is_empty() {
        prefix.extend([
            "--disable-sources".into(),
            globals.disable_sources.join(",").into(),
        ]);
    }
    if prefix.is_empty() {
        return steps;
    }
    steps
        .into_iter()
        .map(|mut step| {
            let mut args = step.args.into_vec();
            args.splice(0..0, prefix.iter().cloned());
            step.args = args.into();
            step
        })
        .collect()
}
