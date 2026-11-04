//! Retained gene-keyed source contracts exercise only the shared row mapper.
//! They add no product command and remain outside the association coverage floor.
use super::{map, raw};
use biomcp_fetch::Reply;
#[test]
#[ignore = "requires retained P21–P23 bodies outside Git"]
fn retained_gene_rows_preserve_identity_taxon_and_row_publications() -> Result<(), String> {
    #[expect(
        clippy::disallowed_methods,
        reason = "Ignored retained-source contract locates external evidence only"
    )]
    let directory = std::env::var_os("BIOMCP_MONARCH_MAPPING_FIXTURES")
        .ok_or("missing retained mapping directory")?;
    for (case, subject, original, total) in [
        ("P21", "HGNC:12687", "NCBIGene:7428", 58),
        ("P22", "HGNC:9588", "NCBIGene:5728", 85),
        ("P23", "HGNC:7765", "NCBIGene:4763", 129),
    ] {
        #[expect(
            clippy::disallowed_methods,
            reason = "Ignored retained-source contract reads bodies kept outside Git"
        )]
        let bytes = std::fs::read(
            std::path::Path::new(&directory).join(format!("{case}-associations.raw")),
        )
        .map_err(|e| e.to_string())?;
        let page: raw::Page = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        page.check()
            .map_err(|e| format!("bad retained page: {e:?}"))?;
        if page.total != total || page.items.len() != 2 {
            return Err(format!("{case} native page changed"));
        }
        for (index, row) in page.items.into_iter().enumerate() {
            let requested = row.object.clone().ok_or("missing object")?;
            let mapped = map::row(row, &requested)
                .map_err(|e| format!("map: {e:?}"))?
                .value
                .association;
            if mapped.subject.id.to_string() != subject
                || mapped
                    .original_subject
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref()
                    != Some(original)
                || mapped
                    .subject_taxon
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref()
                    != Some("NCBITaxon:9606")
            {
                return Err(format!("{case} source identity or taxon changed"));
            }
            let pmid = mapped
                .publications
                .iter()
                .any(|r| r.id.to_string() == "PMID:8317503");
            if case == "P23" && pmid != (index == 1) {
                return Err("NF1 publication moved between rows".into());
            }
        }
    }
    Ok(())
}
