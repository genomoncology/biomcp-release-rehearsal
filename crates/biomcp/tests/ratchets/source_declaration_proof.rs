//! Source policy and ownership violations are rejected by the structural boundary.
use super::{
    Check, check,
    proof_support::{put, tree},
};

#[test]
fn source_registry_accepts_directory_and_file_modules_and_names_missing_entry() -> Check {
    let tree = tree()?;
    let declaration = registry_fixture(tree.path())?;
    check(tree.path())?;
    let auxiliary = "crates/biomcp-adapters/src/directory/other.rs";
    put(
        tree.path(),
        auxiliary,
        &format!("pub static EXTRA: SourceSpec = {declaration};"),
    )?;
    let failure = check(tree.path())
        .err()
        .ok_or("undeclared auxiliary source accepted")?
        .to_string();
    assert!(
        failure.contains("&crate::directory::other::EXTRA"),
        "{failure}"
    );
    put(
        tree.path(),
        "crates/biomcp-adapters/src/directory/decl.rs",
        "pub static DECL: SourceDecl = SourceDecl { spec: &super::SPEC, auxiliary: &[&crate::directory::other::EXTRA] };",
    )?;
    check(tree.path())?;
    put(
        tree.path(),
        auxiliary,
        &format!(
            "pub static EXTRA: SourceSpec = {declaration};\nfn hidden() {{ let other = {declaration}; }}"
        ),
    )?;
    let failure = check(tree.path())
        .err()
        .ok_or("dynamic source policy accepted")?
        .to_string();
    assert!(
        failure.contains("source declarations require named statics"),
        "{failure}"
    );
    put(
        tree.path(),
        auxiliary,
        &format!("pub static EXTRA: SourceSpec = {declaration};"),
    )?;
    put(
        tree.path(),
        "crates/biomcp-adapters/src/missing.rs",
        &format!("pub static SPEC: SourceSpec = {declaration};"),
    )?;
    let failure = check(tree.path())
        .err()
        .ok_or("undeclared single-file source accepted")?
        .to_string();
    assert!(failure.contains("missing.rs"), "{failure}");
    assert!(failure.contains("&crate::missing::SPEC"), "{failure}");
    Ok(())
}

#[test]
fn source_declaration_rejects_wrong_source_and_missing_dispatch() -> Check {
    let tree = tree()?;
    put(
        tree.path(),
        "crates/biomcp-adapters/src/decl.rs",
        "Source::Hgnc => &crate::hgnc::decl::DECL,",
    )?;
    put(
        tree.path(),
        "crates/biomcp-adapters/src/hgnc/mod.rs",
        r#"pub static SPEC: SourceSpec = SourceSpec {
            source: Source::MyGene,
            terms: Terms { name: "Publisher", url: "https://example.org/terms", usage: Open }
        };"#,
    )?;
    put(
        tree.path(),
        "crates/biomcp-adapters/src/hgnc/decl.rs",
        "pub static DECL: SourceDecl = SourceDecl { spec: &super::SPEC, auxiliary: &[] };",
    )?;
    let failure = check(tree.path())
        .err()
        .ok_or("wrong source policy accepted")?
        .to_string();
    assert!(failure.contains("wrong source"), "{failure}");
    put(tree.path(), "crates/biomcp-adapters/src/decl.rs", "")?;
    let failure = check(tree.path())
        .err()
        .ok_or("undispatched source accepted")?
        .to_string();
    assert!(failure.contains("must appear in decl(source)"), "{failure}");
    Ok(())
}

#[test]
fn source_policy_rejects_aliased_and_computed_identity() -> Check {
    for identity in [
        "WRONG_SOURCE",
        "if true { Source::Hgnc } else { Source::MyGene }",
    ] {
        let tree = tree()?;
        put(
            tree.path(),
            "crates/biomcp-adapters/src/hgnc/mod.rs",
            &format!(
                r#"pub static SPEC: SourceSpec = SourceSpec {{
                    source: {identity},
                    terms: Terms {{ name: "Publisher", url: "https://example.org/terms", usage: Open }}
                }};"#
            ),
        )?;
        let failure = check(tree.path())
            .err()
            .ok_or("unrecognized concrete identity accepted")?
            .to_string();
        assert!(
            failure.contains("requires an exact Source variant"),
            "{failure}"
        );
    }
    Ok(())
}

#[test]
fn generic_source_constructor_rejects_unnamed_provider_facts() -> Check {
    for (path, name, base) in [
        ("other.rs", "other", "carrier.base"),
        (
            "decl.rs",
            "component_spec",
            "\"https://unregistered.example.org\"",
        ),
    ] {
        let tree = tree()?;
        put(
            tree.path(),
            &format!("crates/biomcp-adapters/src/{path}"),
            &format!(
                "fn {name}(source: Source, carrier: &SourceSpec, terms: Terms) -> SourceSpec {{\n\
                    SourceSpec {{ source, terms, base: {base},\n\
                    destinations: biomcp_fetch::Destinations::Primary, user_agent: carrier.user_agent,\n\
                    key: KeyUse::None, quota: carrier.quota, deadline: carrier.deadline }}\n}}"
            ),
        )?;
        let failure = check(tree.path())
            .err()
            .ok_or("unnamed provider facts accepted")?
            .to_string();
        assert!(
            failure.contains("source declarations require named statics"),
            "{failure}"
        );
    }
    Ok(())
}

#[test]
fn transport_source_names_have_no_legacy_exceptions() -> Check {
    for file in ["secret.rs", "cache/schema.rs", "bundle/transfer.rs"] {
        super::proof_support::rejects(
            &format!("crates/biomcp-fetch/src/{file}"),
            "const PLANTED_PROVIDER: &str = \"hgnc\";",
            "transport names source hgnc",
        )?;
    }
    Ok(())
}

fn registry_fixture(root: &std::path::Path) -> Result<&'static str, Box<dyn std::error::Error>> {
    let dispatcher = "Source::Directory => &crate::directory::decl::DECL, Source::Single => &crate::single_file::decl::DECL,";
    put(root, "crates/biomcp-adapters/src/decl.rs", dispatcher)?;
    let declaration = r#"SourceSpec {
        source: Source::Directory,
        terms: Terms { name: "Publisher", url: "https://example.org/terms", usage: Open }
    }"#;
    for (path, owner) in [
        ("directory/mod.rs", "directory"),
        ("single_file.rs", "single_file"),
    ] {
        let policy = if owner == "single_file" {
            declaration.replace("Directory", "Single")
        } else {
            declaration.to_owned()
        };
        put(
            root,
            &format!("crates/biomcp-adapters/src/{path}"),
            &format!("pub static SPEC: SourceSpec = {policy};"),
        )?;
        put(
            root,
            &format!("crates/biomcp-adapters/src/{owner}/decl.rs"),
            "pub static DECL: SourceDecl = SourceDecl { spec: &super::SPEC, auxiliary: &[] };",
        )?;
    }
    Ok(declaration)
}
