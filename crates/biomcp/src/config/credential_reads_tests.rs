//! Credential reads are observable only at the settings boundary, before outgoing headers exist.
use super::*;
use std::sync::mpsc;

fn execute(words: &[&str], allowed: Option<SecretName>) -> usize {
    let (send, receive) = mpsc::channel();
    let secrets = secrets_using(None, move |name| {
        assert_eq!(
            Some(name),
            allowed.map(SecretName::environment),
            "forbidden key read"
        );
        send.send(()).unwrap();
        Ok(Some(PathBuf::from("synthetic-settings-key")))
    });
    assert!(!format!("{secrets:?}").contains("API_KEY"));
    let directory = tempfile::tempdir().unwrap();
    let fetch = Fetch::open(FetchConfig {
        cache: Some(directory.path().join("cache.sqlite")),
        cache_owners: vec![biomcp_query::store::OWNER],
        mode: Mode::Offline,
        secrets,
        ..FetchConfig::default()
    })
    .unwrap();
    run(&fetch, words);
    receive.try_iter().count()
}

fn run(fetch: &Fetch, words: &[&str]) -> serde_json::Value {
    let config = tempfile::NamedTempFile::new().unwrap();
    fs::write(config.path(), b"{}").unwrap();
    let mut args = words.iter().map(OsString::from).collect::<Vec<_>>();
    args.extend([
        OsString::from("--config"),
        config.path().as_os_str().to_owned(),
    ]);
    let root = crate::grammar::root().unwrap();
    let parsed = crate::grammar::parse(&root, &args).unwrap();
    let settings = Ok(None);
    let fetch = super::restrict(fetch.clone(), &parsed.globals).unwrap();
    crate::command::execute(
        &root,
        &settings,
        &fetch,
        parsed.action.unwrap(),
        &parsed.globals,
        false,
        crate::command::Encoding::Mcp,
    )
    .unwrap()
    .value()
    .unwrap()
    .clone()
}

#[test]
fn unrelated_and_disabled_commands_never_read_keys() {
    for command in [
        vec!["get", "gene", "HGNC:3236"],
        vec![
            "search",
            "article",
            "EGFR",
            "--source",
            "pubmed",
            "--disable-sources",
            "pubmed",
        ],
        vec!["search", "trial", "--condition", "melanoma"],
        vec![
            "get",
            "variant",
            "clinvar:13961",
            "clinvar",
            "--disable-sources",
            "clinvar",
        ],
        vec![
            "get",
            "variant",
            "chr7:g.140453136A>T",
            "oncokb",
            "--build",
            "GRCh37",
            "--tumor-type",
            "Synthetic tumor",
            "--disable-sources",
            "oncokb",
        ],
        vec![
            "search",
            "trial",
            "--source",
            "nci",
            "--keyword",
            "melanoma",
            "--disable-sources",
            "nci",
        ],
    ] {
        assert_eq!(execute(&command, None), 0);
    }
}

#[test]
fn called_sources_read_only_their_own_environment_key_once() {
    let cases = [
        (
            SecretName::OpenfdaApiKey,
            vec!["get", "diagnostic", "K173492", "--source", "fda-510k"],
        ),
        (
            SecretName::NcbiApiKey,
            vec!["search", "article", "EGFR", "--source", "pubmed"],
        ),
        (
            SecretName::OrcidAccessToken,
            vec!["get", "author", "orcid:0000-0002-4299-3018"],
        ),
        (
            SecretName::NcbiApiKey,
            vec!["get", "variant", "clinvar:13961", "clinvar"],
        ),
        (
            SecretName::SemanticScholarApiKey,
            vec!["get", "author", "semantic-scholar:123"],
        ),
        (
            SecretName::OncokbApiKey,
            vec![
                "get",
                "variant",
                "chr7:g.140453136A>T",
                "oncokb",
                "--build",
                "GRCh37",
                "--tumor-type",
                "Synthetic tumor",
            ],
        ),
        (
            SecretName::NciApiKey,
            vec![
                "search",
                "trial",
                "--source",
                "nci",
                "--keyword",
                "melanoma",
            ],
        ),
    ];
    assert_eq!(
        cases
            .iter()
            .map(|(name, _)| *name)
            .collect::<std::collections::BTreeSet<_>>(),
        biomcp_query::sources::keys()
            .iter()
            .copied()
            // AlphaGenome has source admission but no product section until capture.
            .filter(|name| *name != SecretName::AlphaGenomeApiKey)
            .collect()
    );
    for (name, command) in cases {
        assert_eq!(execute(&command, Some(name)), 1, "{name:?}");
    }
}

#[test]
fn config_keys_override_environment_and_resolve_once_per_owner() {
    for name in biomcp_query::sources::keys() {
        let keys = Some(
            [(
                name.config().to_owned(),
                "synthetic-settings-key".to_owned(),
            )]
            .into(),
        );
        let secrets = secrets_using(keys, |_| panic!("config key read environment"));
        let fetch = Fetch::open(FetchConfig {
            secrets,
            ..FetchConfig::default()
        })
        .unwrap();
        assert!(fetch.has_key(name));
        assert!(fetch.has_key(name));
        assert!(!format!("{fetch:?}").contains("synthetic-settings-key"));
    }
}

#[test]
fn saved_query_pages_never_resolve_credentials() {
    for (recording, words, pointer) in [
        (
            "trial/nci/get",
            vec![
                "get",
                "trial",
                "NCT05136196",
                "diseases",
                "--source",
                "nci",
                "--limit",
                "1",
            ],
            "/next",
        ),
        (
            "author/synthetic",
            vec!["search", "author", "Louis Williams"],
            "/page/pool/id",
        ),
    ] {
        let secrets = secrets_using(None, |_| Ok(Some(PathBuf::from("synthetic-settings-key"))));
        let fetch = Fetch::open(FetchConfig {
            mode: Mode::Replay {
                dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/replies")
                    .join(recording),
            },
            cache_owners: vec![biomcp_query::store::OWNER],
            secrets,
            ..FetchConfig::default()
        })
        .unwrap();
        let first = run(&fetch, &words);
        assert_eq!(first["status"], "complete", "{first}");
        let forbidden = fetch.with_secrets(secrets_using(None, |_| panic!("pool resolved a key")));
        let page = if recording == "trial/nci/get" {
            first
                .pointer(pointer)
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .find_map(|step| {
                    step["args"]
                        .as_array()
                        .filter(|args| args.iter().any(|arg| arg == "--pool"))
                })
                .unwrap()
                .iter()
                .map(|arg| arg.as_str().unwrap())
                .collect::<Vec<_>>()
        } else {
            let mut page = words.clone();
            page.extend(["--pool", first.pointer(pointer).unwrap().as_str().unwrap()]);
            page
        };
        let saved = run(&forbidden, &page);
        assert_eq!(saved["status"], "complete", "{saved}");
        if recording == "trial/nci/get" {
            assert!(saved.pointer("/result/items/0/card").is_some());
        } else {
            assert_eq!(saved.pointer(pointer), first.pointer(pointer));
        }
    }
}

#[test]
fn disabled_author_sources_never_read_credentials() {
    for (source, id) in [
        ("semantic-scholar", "semantic-scholar:123"),
        ("orcid", "orcid:0000-0002-4299-3018"),
    ] {
        for selector in [source, "noncommercial"] {
            let mut commands = vec![vec!["get", "author", id], vec!["author", "papers", id]];
            if source == "semantic-scholar" {
                commands.push(vec!["search", "author", "name"]);
            }
            for mut command in commands {
                command.extend(["--disable-sources", selector]);
                assert_eq!(execute(&command, None), 0);
            }
        }
    }
}

#[test]
fn disabled_openfda_sources_never_read_credentials() {
    for (source, entity, id) in [
        (
            "openfda-label",
            "drug",
            "00000000-0000-0000-0000-000000000001",
        ),
        ("drugsfda", "drug", "NDA021335"),
        ("faers", "adverse-event", "10000001"),
        ("maude", "adverse-event", "10000001"),
        ("fda-pma", "diagnostic", "P150013"),
        ("fda-510k", "diagnostic", "K173492"),
    ] {
        let command = [
            "get",
            entity,
            id,
            "--source",
            source,
            "--disable-sources",
            source,
        ];
        assert_eq!(execute(&command, None), 0);
        assert_eq!(
            execute(command.get(..5).unwrap(), Some(SecretName::OpenfdaApiKey)),
            1
        );
    }
    assert_eq!(execute(&["sync", "fda-cdx"], None), 0);
}
