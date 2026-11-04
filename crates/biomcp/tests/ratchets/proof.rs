#[path = "helper_owner_proof.rs"]
mod helper_owner_proof;
use super::proof_support::{put, rejects, tree};
use super::{Check, check};
use std::fs;

#[test]
fn isolated_trees_reject_structural_violations() -> Check {
    let valid = tree()?;
    check(valid.path())?;
    let cases = [
        (
            "crates/biodata/Cargo.toml",
            "[package]\nname='biodata'\n[lints]\nworkspace=true\n[dependencies]\nbiomcp-types={path='../biomcp-types'}\n",
            "forbidden dependencies dependency",
        ),
        (
            "crates/biodata/Cargo.toml",
            "[lints]\nworkspace=true\n[build-dependencies]\nhttp={package='ureq',version='3'}\n",
            "forbidden build-dependencies dependency",
        ),
        (
            "crates/biodata/Cargo.toml",
            "[lints]\nworkspace=true\n[target.'cfg(unix)'.dependencies]\nhttp.workspace=true\n",
            "forbidden dependencies dependency",
        ),
        (
            "crates/biodata/Cargo.toml",
            "[package]\nname='biodata'\n",
            "workspace lint inheritance",
        ),
        (
            "crates/biodata/src/lib.rs",
            "#[test]\nfn hidden() {}",
            "inline test",
        ),
        (
            "crates/biodata/src/lib.rs",
            "#[expect(dead_code, reason=\"unused\")]\nfn unused() {}",
            "suppressions forbidden",
        ),
        (
            "crates/biodata/src/lib.rs",
            "#[expect(clippy::panic, reason=\"test\")]\nfn unused() {}",
            "lint exception",
        ),
        (
            "crates/biomcp-adapters/src/example/raw.rs",
            "#[serde(default)]\nstruct Raw;",
            "raw serde default",
        ),
        (
            "crates/biodata/src/lib.rs",
            "// Quirk: missing.yaml\n",
            "Quirk requires",
        ),
        (
            "crates/biomcp-query/src/lib.rs",
            "use rusqlite as sql;",
            "SQLite references",
        ),
        (
            "crates/biomcp-adapters/src/lib.rs",
            "SourceKey::declare(\"provider\");",
            "quota declarations",
        ),
        (
            "crates/biomcp-adapters/src/quotas.rs",
            "SourceKey::declare(\"provider\");\nSourceKey::declare(\"provider\");",
            "duplicate source key",
        ),
    ];
    for (path, content, rule) in cases {
        rejects(path, content, rule)?;
    }
    rejects(
        "crates/biodata/src/lib.rs",
        &"// line\n".repeat(601),
        "file line limit",
    )?;
    rejects(
        "crates/biomcp-adapters/src/example/map.rs",
        &"// line\n".repeat(301),
        "file line limit",
    )?;
    Ok(())
}

#[test]
fn quirks_require_cases_inside_case_directory() -> Check {
    let tree = tree()?;
    put(
        tree.path(),
        "tests/cases/gene/example.yaml",
        "case: example",
    )?;
    let source = "crates/biodata/src/lib.rs";
    put(tree.path(), source, "// Quirk: gene/example.yaml\n")?;
    check(tree.path())?;
    put(tree.path(), "tests/outside.yaml", "case: outside")?;
    for reference in [
        "../outside.yaml".to_owned(),
        tree.path().join("tests/outside.yaml").display().to_string(),
    ] {
        put(tree.path(), source, &format!("// Quirk: {reference}\n"))?;
        let failure = check(tree.path()).expect_err("outside case accepted");
        assert!(failure.to_string().contains("Quirk requires"));
    }
    Ok(())
}

#[test]
fn static_quota_references_are_not_quota_declarations() -> Check {
    let tree = tree()?;
    put(
        tree.path(),
        "crates/biodata/src/lib.rs",
        "pub struct SourceSpec { pub quota: &'static Quota }\n",
    )?;
    check(tree.path())?;
    rejects(
        "crates/biodata/src/lib.rs",
        "static QUOTA: Quota = todo!();",
        "quota declarations",
    )?;
    Ok(())
}

#[test]
fn case_parser_stays_outside_runtime_dependencies() -> Check {
    rejects(
        "crates/biomcp/Cargo.toml",
        "[package]\nname='biomcp'\n[lints]\nworkspace=true\n[dependencies]\nserde-saphyr='2'\n",
        "forbidden dependencies dependency",
    )
}

#[test]
fn rejects_unlisted_integration_test_folders() -> Check {
    rejects(
        "crates/biomcp-adapters/tests/unlisted/hidden.rs",
        "#[test]\nfn hidden() {}",
        "test outside listed locations",
    )
}

#[test]
fn rejects_conditional_integration_test_attributes() -> Check {
    rejects(
        "crates/biomcp-adapters/tests/unlisted/conditional.rs",
        "#[cfg_attr(test, test)]\nfn hidden() {}",
        "test outside listed locations",
    )
}

#[test]
fn source_without_publisher_terms_is_rejected() -> Check {
    rejects(
        "crates/biomcp-adapters/src/planted.rs",
        "pub static SPEC: SourceSpec = SourceSpec { source: Source::Hgnc };",
        "publisher terms",
    )
}

#[test]
fn test_files_obey_the_file_line_limit() -> Check {
    let tree = tree()?;
    let path = "crates/biodata/tests/nested/size.rs";
    put(tree.path(), path, &"// line\n".repeat(600))?;
    check(tree.path())?;
    rejects(path, &"// line\n".repeat(601), "file line limit")
}

#[test]
fn source_outcomes_require_registry_terms() -> Check {
    let tree = tree()?;
    put(
        tree.path(),
        "crates/biomcp-query/src/example.rs",
        "fn response() { SourceOutcome { source: Source::Hgnc } }",
    )?;
    let error = check(tree.path())
        .err()
        .ok_or("unlabeled source outcome accepted")?;
    assert!(
        error
            .to_string()
            .contains("source outcomes require registry terms")
    );
    Ok(())
}

#[test]
fn rust_lines_obey_width_limit() -> Check {
    for path in ["crates/biodata/src/lib.rs", "crates/biodata/tests/width.rs"] {
        let tree = tree()?;
        put(tree.path(), path, &format!("//{}\n", "x".repeat(158)))?;
        check(tree.path())?;
        rejects(
            path,
            &format!("//{}\n", "x".repeat(159)),
            "line width limit",
        )?;
    }
    Ok(())
}

#[test]
fn request_reason_mapping_cannot_be_copied() -> Check {
    let root = tempfile::tempdir()?;
    let source = root.path().join("crates/biomcp-query/src");
    fs::create_dir_all(&source)?;
    for copied in [
        "fn reason(error: RequestError) -> Reason { match error { RequestError::Source(e) => e.reason, _ => Reason::Unavailable } }",
        "match reply { Err(RequestError::Source(e)) => Section::unavailable(source, e.reason), Err(_) => \
        Section::unavailable(source, Reason::Unavailable) }",
        "match error { RequestError::Source(SourceError { reason, .. }) => reason, _ => Reason::Unavailable }",
        "let reason = if let biomcp_fetch::RequestError::Source(e) = error { e.reason } else { Reason::Unavailable };",
        "fn reason(error: RequestError) -> Reason { if let RequestError::Source(e) = error { return e.reason; } Reason::Unavailable }",
        "if let RequestError::Source(e) = error { Some(e.reason) } else { None }",
        "fn reason(error: RequestError) -> Option<Reason> { if let RequestError::Source(e) = error { return Some(e.reason); } None }",
    ] {
        fs::write(source.join("section.rs"), copied)?;
        if super::reasons::check(root.path()).is_ok() {
            return Err("a copied request reason mapping passed the repository gate".into());
        }
    }
    fs::write(
        source.join("section.rs"),
        "if matches!(error, RequestError::Source(e) if e.reason == Reason::NotFound) { absent() }\nif let \
            RequestError::Source(e) = error { if e.reason == Reason::NotFound { absent() } }",
    )?;
    super::reasons::check(root.path())
}

#[test]
fn generated_exception_cannot_hide_product_code() -> Check {
    let tree = tree()?;
    let path = "crates/biomcp-adapters/src/alphagenome/proto.rs";
    let wrapper = r#"#![expect(dead_code, reason="pinned generation")]
include!("generated/mod.rs");
"#;
    put(tree.path(), path, wrapper)?;
    check(tree.path())?;
    rejects(
        path,
        &(wrapper.to_owned() + "fn hidden() {}"),
        "generated exception requires only",
    )?;
    for modified in [
        wrapper.replace("dead_code", "dead_code, unused_variables"),
        wrapper.replace("#![expect", "#[expect"),
        format!("fn hidden() {{}}\n{wrapper}"),
    ] {
        rejects(path, &modified, "generated exception requires only")?;
    }
    rejects(
        path,
        &wrapper.replace("dead_code", "unused_imports"),
        "suppressions forbidden",
    )?;
    rejects(
        "crates/biodata/src/lib.rs",
        wrapper,
        "suppressions forbidden",
    )?;
    Ok(())
}

#[test]
fn new_optional_drop_requires_its_own_classification() -> Check {
    let tree = tree()?;
    let path = "crates/biomcp-adapters/src/new_source/details.rs";
    put(
        tree.path(),
        path,
        "fn map() { /* .ok() */ let native = \".ok()\"; }",
    )?;
    check(tree.path())?;
    for source in [
        "fn map() { let title = input.parse().ok(); }",
        "fn map() { let title = input.parse() . ok ( ); }",
    ] {
        rejects(path, source, "classify new optional-value drop")?;
        rejects(
            "crates/biomcp-query/src/new_query.rs",
            source,
            "classify new optional-value drop",
        )?;
    }
    Ok(())
}

#[test]
fn test_fixture_paths_have_one_owner() -> Check {
    for read in [
        "env!(\"CARGO_MANIFEST_DIR\")",
        "env! { \"CARGO_MANIFEST_DIR\" }",
        "option_env!(\"CARGO_MANIFEST_DIR\")",
        "std::env::var(\"CARGO_MANIFEST_DIR\")",
        "std::env::var_os(\"CARGO_MANIFEST_DIR\")",
    ] {
        rejects(
            "crates/biomcp/tests/runner/planted.rs",
            &format!("fn fixture() {{ let root = {read}; }}"),
            "fixture paths belong in shared support/fixtures.rs",
        )?;
    }
    Ok(())
}

#[test]
fn audit_rejects_owned_underscores_and_old_tags_but_allows_native_values() {
    for text in [
        "#[serde(tag = \"annotationkind\")] enum Pgx {}",
        "#[serde(rename_all = \"snake_case\")] enum Owned {}",
        "wire_enum! { Reason { NoKey => \"no_key\" } }",
    ] {
        assert!(
            super::spellings::audit(&super::source::tokens(text), true).is_err(),
            "{text}"
        );
    }
    assert!(
        super::spellings::audit(
            &super::source::tokens("wire_enum! { Native { Value => \"source_native\" } }"),
            true
        )
        .is_ok()
    );
}

#[test]
fn enum_return_types_are_not_macro_declarations() {
    assert!(
        super::spellings::audit(
            &super::source::tokens(
                "fn kind() -> ArticleMatchKind { match self { X => ArticleMatchKind::Doi } }"
            ),
            true
        )
        .is_ok()
    );
}

#[test]
fn namespaced_error_codes_are_exempt_by_type() {
    for name in ["ErrorCode", "ProblemCode"] {
        for literal in ["usage.future_name", "wire_enum.unknown"] {
            let source = format!("wire_enum! {{ {name} {{ New => {literal:?} }} }}");
            assert!(super::spellings::audit(&super::source::tokens(&source), true).is_ok());
            let ordinary = source.replace(name, "Reason");
            assert!(super::spellings::audit(&super::source::tokens(&ordinary), true).is_err());
        }
    }
}

#[test]
fn crate_ownership_keeps_biodata_values_and_audits_biomcp_values() -> Check {
    for source in [
        "#[serde(rename_all = \"snake_case\")] enum Owned {}",
        "wire_enum! { DrugRecallCountUnit { Value => \"recall_records\" } }",
    ] {
        let tree = tree()?;
        put(tree.path(), "crates/biodata/src/lib.rs", source)?;
        check(tree.path())?;
        rejects("crates/biomcp-types/src/lib.rs", source, "kebab-case")?;
    }
    Ok(())
}

#[test]
fn planted_error_signature_and_exit_literals_are_rejected() -> Check {
    super::errors::signatures(
        "fn valid() -> Result<(), Failure> { Ok(()) } ExitCode::from(Status::Local.code())",
    )?;
    for planted in [
        "fn bad() -> Result<Vec<u8>, ()> {}",
        "fn bad() -> Result<Vec<u8>, (),> {}",
        "impl From<()> for Failure {}",
        "impl From<(),> for Failure {}",
        "ExitCode::from(5)",
    ] {
        assert!(
            super::errors::signatures(planted).is_err(),
            "planted violation must fail: {planted}"
        );
    }
    Ok(())
}

#[test]
fn planted_undeclared_display_code_is_rejected() -> Check {
    assert!(
        super::errors::display_codes("One => \"same.error\", Two => \"same.error\"", "").is_err(),
        "duplicate declared codes must fail"
    );
    let declaration = "Known => \"known.error\"";
    super::errors::display_codes(
        declaration,
        "impl Display for Known { f.write_str(\"known.error\") }",
    )?;
    assert!(
        super::errors::display_codes(
            declaration,
            "impl Display for Stray { f.write_str(\"stray.error\") }"
        )
        .is_err(),
        "an undeclared display code must fail"
    );
    Ok(())
}

#[test]
fn adapter_drop_forms_and_file_rules_require_pending_identity() -> Check {
    let root = tempfile::tempdir()?;
    let directory = root.path().join("crates/biomcp-adapters/src/example");
    fs::create_dir_all(&directory)?;
    let path = directory.join("details.rs");
    for source in [
        "fn value() { let native = input.parse().ok(); }",
        "fn value() { if let Ok(native) = input.parse() { use_value(native); } }",
        "fn value() { let Ok(native) = input.parse() else { return; }; }",
        "fn value() { input.filter_map(|v| v.parse().ok()); }",
        "fn value() { input.parse::<u64>().unwrap_or_default(); }",
        "fn value() { let parsed = input.parse::<u64>(); parsed.unwrap_or_default(); }",
        "fn value() { serde_json::from_value::<u64>(input).unwrap_or_default(); }",
        "fn value() { serde_json::from_reader::<_, u64>(input).unwrap_or_default(); }",
        "#[serde(default)] struct Raw;",
        "#[serde(deny_unknown_fields)] struct Raw;",
    ] {
        fs::write(&path, source)?;
        let failure = super::source::check(root.path())
            .expect_err("planted adapter violation passed")
            .to_string();
        assert!(failure.contains("example/details.rs"), "{failure}");
    }
    fs::write(&path, "// line\n".repeat(301))?;
    assert!(
        super::source::check(root.path())
            .expect_err("301-line adapter passed")
            .to_string()
            .contains("file line limit")
    );
    fs::write(&path, "// line\n".repeat(300))?;
    super::source::check(root.path())?;
    fs::write(
        &path,
        "fn value() { /* .ok() */ let note = \"if let Ok(v) .ok()\"; }",
    )?;
    super::source::check(root.path())
}

#[test]
fn inventory_allows_shrinking_and_reclassification_but_refuses_new_identities() -> Check {
    super::proof_support::check_inventory_mutations()
}

#[test]
fn provider_codecs_stay_outside_transport_and_query() -> Check {
    rejects(
        "crates/biomcp-fetch/src/grpc/worker.rs",
        "fn request() { let alphagenome = 1; consume(alphagenome); }",
        "transport names source alphagenome",
    )?;
    rejects(
        "crates/biomcp-query/Cargo.toml",
        "[lints]\nworkspace=true\n[dependencies]\nprost='0.14'\n",
        "forbidden dependencies dependency",
    )
}

#[test]
fn discovery_rejects_unknown_missing_and_unfeatured_trials() -> Check {
    let features =
        "known: A caller receives a known answer.\nother: A caller receives another answer.\n";
    super::feature_contract::validate(features, ["known/case", "other/trial"], true)?;
    for (names, expected) in [
        (vec!["unknown/case", "other/trial"], "unlisted feature"),
        (vec!["known/case"], "features without tests"),
        (vec!["unfeatured", "other/trial"], "trial lacks feature"),
    ] {
        let failure = super::feature_contract::validate(features, names, true)
            .err()
            .ok_or("invalid discovery was accepted")?;
        assert!(failure.contains(expected), "{failure}");
    }
    Ok(())
}

#[test]
fn tool_dispatch_stays_outside_cases() -> Check {
    rejects(
        "crates/biomcp/tests/cases.rs",
        r##"fn main() { if arg == r#"--planted-tool-mode"# { tool(); } }"##,
        "tool modes belong in tools.rs",
    )
}

#[test]
fn mapped_constructions_require_their_own_refusal_decision() -> Check {
    super::proof_support::check_mapped_refusal_decisions()
}

#[test]
fn only_owned_private_refusal_defaults_bypass_native_ceiling() -> Check {
    super::proof_support::private_refusal_defaults()
}

#[test]
fn structural_diagnostics_keep_independent_categories() -> Check {
    super::proof_support::independent_structural_failures()
}

#[test]
fn adapter_mapping_helpers_require_one_owner() -> Check {
    helper_owner_proof::check()
}

#[test]
fn reject_config_reads_and_tree_builds_in_commands() -> Check {
    super::process_setup::reject_config_reads_and_tree_builds_in_commands()
}

#[test]
fn module_shape_rejects_new_source_debt() -> Check {
    let tree = tree()?;
    let root = tree.path();
    let parent = "crates/biodata/src/lib.rs";
    let leaf = "crates/biodata/src/value.rs";
    put(
        root,
        parent,
        "//! Values.\n#[cfg(any())] pub(crate) mod value;\npub use value::{Value};\n",
    )?;
    put(root, leaf, "//! A value.\npub struct Value;\n")?;
    super::module_shape::limits(root, 0, 0)?;
    put(root, "crates/biodata/src/new.rs", "pub struct New;\n")?;
    let failure = super::module_shape::limits(root, 0, 0)
        .expect_err("undocumented file must fail")
        .to_string();
    assert!(
        failure.contains("missing first-line module docs"),
        "{failure}"
    );
    fs::remove_file(root.join("crates/biodata/src/new.rs"))?;
    for item in [
        "pub fn value() {}",
        "pub struct Value;",
        "const VALUE: u8 = 1;",
        "mod nested {}",
        "macro_rules! value { () => {} }",
    ] {
        put(root, parent, &format!("//! Values.\nmod value;\n{item}\n"))?;
        let failure = super::module_shape::limits(root, 0, 0)
            .expect_err("parent logic must fail")
            .to_string();
        assert!(failure.contains("parent modules with logic"), "{failure}");
    }
    for literal in [
        "'{'",
        "'}'",
        "'\"'",
        "'\\''",
        "'\\\\'",
        "r#\"\" }\"#",
        "br##\"\" }\"##",
    ] {
        put(
            root,
            parent,
            &format!("//! Values.\nfn value() {{ let _ = {literal}; }}\nmod value;\n"),
        )?;
        let failure = super::module_shape::limits(root, 0, 0)
            .expect_err("quoted delimiters must not hide a parent")
            .to_string();
        assert!(failure.contains("parent modules with logic"), "{failure}");
    }
    put(
        root,
        parent,
        "/*! Values. */\nmod value;\nextern crate core;\nuse core::{fmt, hash};\n",
    )?;
    super::module_shape::limits(root, 0, 0)?;
    put(
        root,
        parent,
        "#![doc = include_str!(\"../README.md\")]\nmod value;\n",
    )?;
    super::module_shape::limits(root, 0, 0)?;
    for prefix in ["/// Outer item docs.", "//!", "\n//! Late module docs."] {
        put(root, leaf, &format!("{prefix}\npub struct Value;\n"))?;
        assert!(
            super::module_shape::limits(root, 0, 0).is_err(),
            "invalid first-line module doc passed: {prefix:?}"
        );
    }
    Ok(())
}
