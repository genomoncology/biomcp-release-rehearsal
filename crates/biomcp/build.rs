//! Embed the checked-in schema artifacts by their canonical identities.
#![expect(
    clippy::disallowed_methods,
    reason = "Build scripts read Cargo environment and emit Cargo directives"
)]
use std::{collections::BTreeMap, env, error::Error, fs, path::PathBuf};

#[path = "build/front_map.rs"]
mod front_map;
#[path = "build/schema_metadata.rs"]
mod schema_metadata;

fn main() -> Result<(), Box<dyn Error>> {
    embed_build()?;
    let directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../biomcp-types/schemas");
    println!("cargo:rerun-if-changed={}", directory.display());
    let mut schemas = BTreeMap::new();
    for entry in fs::read_dir(&directory)? {
        let path = entry?.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let text = fs::read_to_string(&path)?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        let id = value
            .get("$id")
            .and_then(serde_json::Value::as_str)
            .ok_or("schema artifact requires a string $id")?
            .to_owned();
        if schemas.insert(id, text).is_some() {
            return Err("duplicate schema artifact $id".into());
        }
    }
    schema_metadata::write(&schemas, &PathBuf::from(env::var("OUT_DIR")?))?;
    let entries = schemas
        .iter()
        .map(|(id, text)| format!("({id:?}, {text:?}),\n"))
        .collect::<String>();
    let output = PathBuf::from(env::var("OUT_DIR")?).join("schemas.rs");
    fs::write(output, format!("&[\n{entries}]\n"))?;
    let map = front_map::generate(&schemas)?;
    println!("cargo:rerun-if-env-changed=BIOMCP_FRONT_MAP_CHARACTER_LIMIT");
    let character_limit = env::var("BIOMCP_FRONT_MAP_CHARACTER_LIMIT")
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(front_map::DEFAULT_CHARACTER_LIMIT);
    if map.chars().count() > character_limit {
        return Err(format!(
            "front map exceeds {} characters: {}",
            character_limit,
            map.chars().count()
        )
        .into());
    }
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("front-map.txt"),
        &map,
    )?;
    let typed_map = front_map::generate_typed(&schemas)?;
    if typed_map.chars().count() > 4096 {
        return Err(format!(
            "typed front map exceeds 4096 characters: {}",
            typed_map.chars().count()
        )
        .into());
    }
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("typed-front-map.txt"),
        typed_map,
    )?;
    embed_skills(&map)?;
    embed_examples()
}

const EXAMPLE_CASES: &[(&str, usize)] = &[
    ("biomarker/synonym.yaml", 0),
    ("biomarker/code.yaml", 0),
    ("organization/name.yaml", 0),
    ("organization/city-state.yaml", 0),
    ("gene/hgnc-sync.yaml", 0),
    ("gene/sections/clingen-sync.yaml", 0),
    ("gene/sections/gencc.yaml", 0),
    ("store/clean.yaml", 0),
    ("author/search.yaml", 0),
    ("author/source-alias.yaml", 0),
    ("author/ordered-cards.yaml", 0),
    ("author/repeated-input.yaml", 0),
    ("author/papers.yaml", 0),
    ("author/papers-details.yaml", 0),
    ("cell-line/assay-examples.yaml", 0),
    ("cell-line/assay-examples.yaml", 1),
    ("cell-line/examples.yaml", 0),
    ("cell-line/examples.yaml", 1),
    ("cell-line/examples.yaml", 2),
    ("cell-line/examples.yaml", 3),
    ("study/examples.yaml", 0),
    ("study/examples.yaml", 1),
    ("study/examples.yaml", 2),
    ("study/examples.yaml", 3),
    ("study/examples.yaml", 4),
    ("study/examples.yaml", 5),
    ("gwas/examples.yaml", 0),
    ("gwas/examples.yaml", 1),
    ("gwas/get-examples.yaml", 0),
    ("gwas/get-examples.yaml", 1),
    ("protein/cards.yaml", 0),
    ("protein/gene-crossrefs.yaml", 0),
    ("protein/cards.yaml", 1),
    ("protein/paging.yaml", 0),
    ("protein/examples.yaml", 0),
    ("protein/examples.yaml", 1),
    ("protein/examples.yaml", 2),
    ("protein/examples.yaml", 3),
    ("protein/examples.yaml", 4),
    ("protein/examples.yaml", 5),
    ("protein/examples.yaml", 6),
    ("protein/examples.yaml", 7),
    ("protein/examples.yaml", 8),
    ("protein/examples.yaml", 9),
    ("protein/examples.yaml", 10),
    ("protein/examples.yaml", 11),
    ("protein/examples.yaml", 12),
    ("protein/examples.yaml", 13),
    ("phenotype/examples.yaml", 0),
    ("phenotype/examples.yaml", 1),
    ("phenotype/examples.yaml", 2),
    ("phenotype/examples.yaml", 3),
    ("pathway/examples.yaml", 0),
    ("pathway/examples.yaml", 1),
    ("pathway/examples.yaml", 2),
    ("pathway/examples.yaml", 3),
    ("pathway-sections/examples.yaml", 0),
    ("pathway-sections/examples.yaml", 1),
    ("pathway-sections/examples.yaml", 2),
    ("pathway-sections/examples.yaml", 3),
    ("pathway/sync-examples.yaml", 0),
    ("pathway/sync-examples.yaml", 1),
    ("pgx/examples.yaml", 0),
    ("pgx/examples.yaml", 1),
    ("pgx/examples.yaml", 2),
    ("pgx/examples.yaml", 3),
    ("diagnostic/recall.yaml", 0),
    ("diagnostic/recall.yaml", 1),
    ("diagnostic/recall.yaml", 2),
    ("diagnostic/fda-cdx.yaml", 0),
    ("diagnostic/fda-cdx.yaml", 3),
    ("diagnostic/who.yaml", 0),
    ("diagnostic/who.yaml", 2),
    ("diagnostic/who.yaml", 1),
    ("diagnostic/gtr.yaml", 0),
    ("diagnostic/gtr.yaml", 9),
    ("diagnostic/gtr.yaml", 1),
    ("diagnostic/gtr.yaml", 4),
    ("diagnostic/gtr.yaml", 5),
    ("adverse-event/examples.yaml", 0),
    ("adverse-event/examples.yaml", 1),
    ("adverse-event/maude-filters.yaml", 0),
    ("adverse-event/get-examples.yaml", 0),
    ("adverse-event/get-examples.yaml", 1),
    ("article/pdf/synthetic.yaml", 0),
    ("article/pdf/synthetic.yaml", 1),
    ("drug/ema.yaml", 1),
    ("drug/ema.yaml", 2),
    ("drug/ema.yaml", 3),
    ("drug/ema.yaml", 4),
    ("drug/who-sync.yaml", 0),
    ("drug/who-sync.yaml", 1),
    ("drug/ema-sync.yaml", 0),
    ("drug/ema-sync.yaml", 1),
    ("drug/ddinter.yaml", 0),
    ("drug/ddinter.yaml", 2),
    ("disease/examples.yaml", 0),
    ("disease/examples.yaml", 1),
    ("disease/examples.yaml", 2),
    ("disease/examples.yaml", 3),
    ("readme/gene.yaml", 0),
    ("gene/braf.yaml", 0),
    ("gene/search.yaml", 0),
    ("gene/search.yaml", 1),
    ("readme/article.yaml", 0),
    ("article/filters/since.yaml", 0),
    ("article/get/card.yaml", 0),
    ("article/preprint/reader.yaml", 0),
    ("article/fulltext/outline.yaml", 0),
    ("article/fulltext/outline.yaml", 1),
    ("article/supplements/workbook.yaml", 0),
    ("article/supplements/protein-change.yaml", 0),
    ("article/citations/citing.yaml", 0),
    ("article/edges/window.yaml", 0),
    ("article/citations/references.yaml", 0),
    ("article/edges/references.yaml", 0),
    ("article/evidence/source.yaml", 0),
    ("article/evidence/source.yaml", 1),
    ("article/recommendations/single.yaml", 0),
    ("article/recommendations/multi.yaml", 0),
    ("readme/trial.yaml", 0),
    ("trial/nci-facility-native.yaml", 0),
    ("trial/intervention.yaml", 0),
    ("trial/or-options.yaml", 0),
    ("trial/get/NCT04303780.yaml", 0),
    ("trial/documents.yaml", 0),
    ("trial/document-examples.yaml", 0),
    ("trial/document-examples.yaml", 1),
    ("variant/assertions-domains.yaml", 0),
    ("variant/assertions-erepo-current.yaml", 0),
    ("variant/search-L858R.yaml", 0),
    ("variant/search-Arg117Gly.yaml", 0),
    ("variant/help-assertions-clinvar.yaml", 0),
    ("variant/help-assertions-erepo.yaml", 0),
    ("variant/help-population.yaml", 0),
    ("variant/help-population.yaml", 1),
    ("variant/help-normalize-mutalyzer.yaml", 0),
    ("variant/help-normalize-variantvalidator.yaml", 0),
    ("variant/recurrence-source-query.yaml", 0),
    ("variant/help-recurrence.yaml", 0),
];

const GUIDE_CASES: &[(&str, usize)] = &[
    ("variant/search-coding-flcn.yaml", 0),
    ("drug/label-discovery.yaml", 0),
    ("drug/label-controls.yaml", 1),
    ("trial/or-options.yaml", 1),
    ("trial/wider-zero.yaml", 0),
    ("trial/wider-zero.yaml", 1),
    ("article/filters/since.yaml", 1),
    ("gene/sections/clingen-validity.yaml", 0),
    ("gene/band.yaml", 0),
    ("gene/band.yaml", 1),
    ("gene/p16.yaml", 0),
    ("gene/search.yaml", 2),
    ("gene/sections/expression-mir21.yaml", 0),
    ("gene/sections/pathways-mir21.yaml", 0),
    ("variant/search-region.yaml", 0),
    ("phenotype/worked-headache.yaml", 0),
    ("phenotype/worked-headache.yaml", 1),
];

fn embed_examples() -> Result<(), Box<dyn Error>> {
    let cases = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../tests/cases");
    let mut examples = BTreeMap::<String, Vec<Vec<String>>>::new();
    let mut witnesses = Vec::new();
    for (entries, help) in [(EXAMPLE_CASES, true), (GUIDE_CASES, false)] {
        for &(file, index) in entries {
            let path = cases.join(file);
            println!("cargo:rerun-if-changed={}", path.display());
            let case: serde_json::Value = serde_saphyr::from_str(&fs::read_to_string(path)?)?;
            let steps = case
                .get("steps")
                .and_then(serde_json::Value::as_array)
                .ok_or("case steps required")?;
            let run: Vec<String> = serde_json::from_value(
                steps
                    .get(index)
                    .and_then(|step| step.get("run"))
                    .ok_or("missing example step")?
                    .clone(),
            )?;
            let command = run
                .get(..2)
                .ok_or("example needs a verb and entity")?
                .join(" ");
            if help {
                examples.entry(command).or_default().push(run);
            }
            witnesses.push((file, index));
        }
    }
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("examples.json"),
        serde_json::to_vec(&examples)?,
    )?;
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("example-cases.json"),
        serde_json::to_vec(&witnesses)?,
    )?;
    Ok(())
}

fn embed_build() -> Result<(), Box<dyn Error>> {
    println!(
        "cargo:rustc-env=BIOMCP_BUILD_TARGET={}",
        env::var("TARGET")?
    );
    println!("cargo:rerun-if-env-changed=BIOMCP_BUILD_REVISION");
    let revision = match env::var("BIOMCP_BUILD_REVISION") {
        Ok(revision) => revision,
        Err(env::VarError::NotPresent) => git_revision()?,
        Err(error) => return Err(error.into()),
    };
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("build revision must be a full Git commit".into());
    }
    println!("cargo:rustc-env=BIOMCP_BUILD_COMMIT={revision}");
    Ok(())
}
fn git_revision() -> Result<String, Box<dyn Error>> {
    use std::process::Command;
    let commit = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !commit.status.success() {
        return Err("cannot resolve build commit".into());
    }
    for name in ["HEAD", "refs/heads"] {
        let output = Command::new("git")
            .args(["rev-parse", "--git-path", name])
            .output()?;
        if !output.status.success() {
            return Err("cannot resolve build revision inputs".into());
        }
        println!(
            "cargo:rerun-if-changed={}",
            String::from_utf8(output.stdout)?.trim()
        );
    }
    Ok(String::from_utf8(commit.stdout)?.trim().to_owned())
}

fn embed_skills(map: &str) -> Result<(), Box<dyn Error>> {
    let directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../skills");
    println!("cargo:rerun-if-changed={}", directory.display());
    let mut assets = BTreeMap::new();
    skill_files(&directory, &directory, &mut assets)?;
    let guide = assets.get_mut("SKILL.md").ok_or("missing skill guide")?;
    if guide.matches("{{front-map}}").count() != 1 {
        return Err("skill template requires exactly one front-map marker".into());
    }
    if guide.len() > 4096 {
        return Err("agent skill template exceeds 4 KB".into());
    }
    *guide = guide.replace("{{front-map}}", map);
    let entries = assets
        .iter()
        .map(|(name, text)| format!("({name:?}, {text:?}),\n"))
        .collect::<String>();
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("skills.rs"),
        format!("&[\n{entries}]\n"),
    )?;
    Ok(())
}
fn skill_files(
    directory: &std::path::Path,
    root: &std::path::Path,
    assets: &mut BTreeMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_symlink() {
            return Err("skill source must not contain symbolic links".into());
        }
        if path.is_dir() {
            skill_files(&path, root, assets)?;
        } else {
            assets.insert(
                path.strip_prefix(root)?
                    .to_str()
                    .ok_or("non-Unicode skill path")?
                    .replace('\\', "/"),
                fs::read_to_string(&path)?,
            );
        }
    }
    Ok(())
}
