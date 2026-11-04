//! Public boundary tables for identifiers, partial dates and coordinate bounds.
use biodata::{HgncId, PartialDate, Position1, Span0};
use serde_json::json;

#[test]
fn hgnc_ids_check_numeric_and_canonical_text_boundaries() {
    for (number, text) in [
        (1, "HGNC:1"),
        (1097, "HGNC:1097"),
        (u32::MAX, "HGNC:4294967295"),
    ] {
        assert_eq!(HgncId::new(number).unwrap().get(), number);
        assert_eq!(HgncId::new(number).unwrap().to_string(), text);
    }
    assert!(HgncId::new(0).is_err());
    for text in [
        "",
        "HGNC:0",
        "HGNC:01",
        "HGNC:+1",
        "HGNC:-1",
        "hgnc:1",
        "HGNC: 1",
        " HGNC:1",
        "HGNC:1\n",
        "HGNC:١",
        "HGNC:4294967296",
        "distinctive-rejected-input",
    ] {
        let error = text.parse::<HgncId>().unwrap_err();
        assert!(!format!("{error:?} {error}").contains("distinctive-rejected-input"));
        let error = serde_json::from_value::<HgncId>(json!(text)).unwrap_err();
        assert!(!error.to_string().contains("distinctive-rejected-input"));
    }
    for value in [json!(1), json!(null), json!(true), json!({})] {
        assert!(serde_json::from_value::<HgncId>(value).is_err());
    }
    let a: HgncId = "HGNC:1".parse().unwrap();
    assert!(std::collections::HashSet::from([a]).contains(&HgncId::new(1).unwrap()));
}

#[test]
fn partial_dates_preserve_precision_and_validate_gregorian_calendar() {
    for (year, month, day, text) in [
        (1, None, None, "0001"),
        (9999, Some(12), Some(31), "9999-12-31"),
        (2000, Some(2), Some(29), "2000-02-29"),
        (2024, Some(2), Some(29), "2024-02-29"),
        (1900, Some(2), Some(28), "1900-02-28"),
        (2023, Some(2), None, "2023-02"),
    ] {
        let value = PartialDate::new(year, month, day).unwrap();
        assert_eq!(
            (value.year(), value.month(), value.day()),
            (year, month, day)
        );
        assert_eq!(value.to_string(), text);
    }
    for args in [
        (0, None, None),
        (10000, None, None),
        (2024, None, Some(1)),
        (2024, Some(0), None),
        (2024, Some(13), None),
        (2024, Some(1), Some(0)),
        (2024, Some(4), Some(31)),
        (1900, Some(2), Some(29)),
        (2023, Some(2), Some(29)),
    ] {
        assert!(PartialDate::new(args.0, args.1, args.2).is_err());
    }
    for text in [
        "",
        "1",
        "0000",
        "10000",
        "2024-2",
        "2024-02-1",
        "2024-02-30",
        "2024-01-01x",
        " 2024",
        "2024\n",
        "2024-",
        "2024--01",
        "２０２４",
        "distinctive-rejected-input",
    ] {
        assert!(text.parse::<PartialDate>().is_err(), "{text}");
        let error = serde_json::from_value::<PartialDate>(json!(text)).unwrap_err();
        assert!(!error.to_string().contains("distinctive-rejected-input"));
    }
    assert!(serde_json::from_value::<PartialDate>(json!(2024)).is_err());
}

#[test]
fn coordinates_enforce_integer_bounds_and_half_open_span_order() {
    for number in [1, u32::MAX] {
        let position = Position1::new(number).unwrap();
        assert_eq!(position.get(), number);
        assert_eq!(serde_json::to_value(position).unwrap(), json!(number));
        assert_eq!(
            serde_json::from_value::<Position1>(json!(number)).unwrap(),
            position
        );
    }
    assert!(Position1::new(0).is_err());
    for value in [
        json!(0),
        json!(-1),
        json!(4294967296_u64),
        json!(1.5),
        json!("1"),
        json!(null),
    ] {
        assert!(serde_json::from_value::<Position1>(value).is_err());
    }
    for (start, end) in [(0, 0), (0, 1), (u32::MAX, u32::MAX), (0, u32::MAX)] {
        let span = Span0::new(start, end).unwrap();
        assert_eq!((span.start(), span.end()), (start, end));
        let value = json!({"start":start,"end":end});
        assert_eq!(serde_json::to_value(span).unwrap(), value);
        assert_eq!(serde_json::from_value::<Span0>(value).unwrap(), span);
    }
    assert!(Span0::new(2, 1).is_err());
    for value in [
        json!({"start":2,"end":1}),
        json!({"start":0}),
        json!({"end":1}),
        json!({"start":-1,"end":1}),
        json!({"start":0,"end":4294967296_u64}),
        json!({"start":"0","end":1}),
        json!([0, 1]),
        json!(null),
    ] {
        assert!(
            serde_json::from_value::<Span0>(value.clone()).is_err(),
            "Span0 admitted {value}"
        );
    }
}

#[path = "value_conventions/sweep.rs"]
mod sweep;

#[test]
fn synonymous_protein_search_roundtrips_canonical_hgvs() -> Result<(), Box<dyn std::error::Error>> {
    let canonical = json!({"gene":"BARD1", "protein":"p.Ala613=", "limit":10, "offset":0});
    for text in ["p.Ala613=", "p.A613="] {
        let query: biomcp_types::VariantSearch =
            serde_json::from_value(json!({"gene":"BARD1", "protein":text}))?;
        assert_eq!(serde_json::to_value(&query)?, canonical);
        assert_eq!(
            biomcp_types::protein_query_forms(text)?,
            ["A613=", "Ala613=", "p.A613=", "p.Ala613="].map(String::from)
        );
    }
    Ok(())
}
