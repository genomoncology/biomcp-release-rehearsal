//! Parser security boundary: verify complete text survives fragmenting, outside the adapter API.
use biomcp_adapters::pmc_oa;
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use std::{path::Path, time::Duration};
#[test]
fn bounded_parts_preserve_original_unicode_rows_and_escapes()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/replies/article/fulltext/synthetic");
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir },
        ..FetchConfig::default()
    })?;
    let leg = fetch.leg(&pmc_oa::SPEC, fetch.deadline(Duration::from_secs(20)));
    let document = pmc_oa::read(&leg, "PMC5606870".parse()?, None)?
        .value
        .ok_or("missing body")?;
    let mut reconstructed = vec![String::new(); document.body.rows.len()];
    for part in &document.body.parts {
        assert!(part.meta.tokens <= 4000);
        for row in &part.rows {
            reconstructed
                .get_mut(row.source_row)
                .ok_or("bad source row")?
                .push_str(&row.text);
        }
    }
    assert_eq!(
        reconstructed,
        document
            .body
            .rows
            .iter()
            .map(|r| r.markdown.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        document
            .body
            .rows
            .iter()
            .any(|r| r.plain == format!("Large {}", "αβ\\\" ".repeat(5000)))
    );
    assert!(reconstructed.contains(&"| AXIN2 row | Value |".to_owned()));
    assert!(reconstructed.contains(&format!("| Large cell | {} |", "β".repeat(13000))));
    assert!(reconstructed.contains(&"Trailing body text.".to_owned()));
    assert_eq!(document.version, Some(12));
    assert!(
        document
            .body
            .rows
            .iter()
            .any(|r| r.references.iter().any(|id| id.as_str() == "B1"))
    );
    assert!(
        document
            .body
            .references
            .iter()
            .any(|r| r.id.as_str() == "B1"
                && r.pmid.is_some_and(|id| id.get() == 1)
                && r.doi.is_some())
    );
    Ok(())
}
