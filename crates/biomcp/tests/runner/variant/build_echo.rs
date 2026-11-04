//! Caller build selection crosses the real command and TLS transport.
use super::{call, myvariant_body};
use serde_json::json;

pub(super) fn check() -> Result<(), String> {
    let (output, requests) = call(
        "myvariant.info",
        myvariant_body,
        &["get", "variant", "chr7:g.140753336A>T", "--build", "GRCh38"],
    )?;
    let [(_, request)] = requests.as_slice() else {
        return Err("expected one exact MyVariant request".into());
    };
    if !request.starts_with(
        "GET \
        /v1/variant/chr7%3Ag.140753336A%3ET?fields=_id%2Cdbsnp.rsid%2Cdbsnp.gene.symbol%2Cclinvar.variant_id%2Csnpeff.ann%2Cchrom%2Chg19%2Chg38%2Cclingen.caid\
        &assembly=hg38 HTTP/1.1",
    ) {
        return Err(format!("exact annotation endpoint/build was not sent: {request}"));
    }
    if output.pointer("/result/candidates/0/identity/representations/0/build")
        != Some(&json!("GRCh38"))
    {
        return Err(format!("source assembly lost: {output}"));
    }
    if output.pointer("/query/build") != Some(&json!("GRCh38"))
        || output.pointer("/query/build_defaulted").is_some()
    {
        return Err(format!(
            "explicit assembly was changed or marked as default: {output}"
        ));
    }
    Ok(())
}
