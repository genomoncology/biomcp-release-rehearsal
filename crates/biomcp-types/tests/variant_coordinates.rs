//! Coordinate parser boundary table; installed matrix owns command behavior.
use biodata::Build;
use biomcp_types::VariantInput;
type Check = Result<(), Box<dyn std::error::Error>>;
#[test]
fn coordinate_inputs_convert_vcf_padding_and_preserve_full_reference_for_validation() -> Check {
    for (input, hgvs, validation) in [
        (
            "13-32315508-GT-G",
            "chr13:g.32315509del",
            "NC_000013.11:g.32315508_32315509delGTinsG",
        ),
        (
            "chr1-55051215-G-GA",
            "chr1:g.55051215_55051216insA",
            "NC_000001.11:g.55051215delGinsGA",
        ),
        (
            "7-140753336-AC-TTT",
            "chr7:g.140753336_140753337delinsTTT",
            "NC_000007.14:g.140753336_140753337delACinsTTT",
        ),
        ("1-1-A-TA", "chr1:g.1delinsTA", "NC_000001.11:g.1delAinsTA"),
        ("1-1-TA-A", "chr1:g.1del", "NC_000001.11:g.1_2delTAinsA"),
    ] {
        let input = VariantInput::new(input)?;
        assert_eq!(input.native_hgvs(), hgvs);
        assert_eq!(
            input.coordinate_validation(Build::GRCh38).as_deref(),
            Some(validation)
        );
    }
    for input in [
        "99-100-A-T",
        "chr0-100-A-T",
        "7-100--T",
        "7-100-A-",
        "7-100-N-T",
        "7-100-AC-AC",
        "7-18446744073709551615-AA-T",
    ] {
        assert!(
            VariantInput::new(input).is_err(),
            "invalid coordinate: {input}"
        );
    }
    assert_eq!(VariantInput::new("M-1-A-T")?.source_id(), "chrMT:g.1A>T");
    assert_eq!(
        VariantInput::new("chrM:g.1A>T")?.source_id(),
        "chrMT:g.1A>T"
    );
    assert!(
        VariantInput::new("7-140753336-A-T")?
            .coordinate_validation(Build::GRCh38)
            .is_none()
    );
    assert_eq!(
        VariantInput::new("7-117559590-ATCT-A")?
            .coordinate_validation(Build::GRCh37)
            .as_deref(),
        Some("NC_000007.13:g.117559590_117559593delATCTinsA")
    );
    Ok(())
}
