use super::{Check, rust_files, source};
use std::{fs, path::Path};

// One authoritative list; architecture 10.4 must reproduce each path and reason.
const ALLOWED: &[(&str, &str)] = &[
    (
        "crates/biomcp/src/mcp/http/load_benchmark.rs",
        "Measure explicitly invoked bounded HTTP load against one owned service and Fetch.",
    ),
    (
        "crates/biomcp/tests/ratchets/source_declaration_proof.rs",
        "Plant source declaration omissions, identity mismatches, unnamed provider facts and transport source literals at the structural boundary.",
    ),
    (
        "crates/biomcp-adapters/tests/source_declarations.rs",
        "Check unique declared bundle command names and their stable public discovery order through the adapter registry.",
    ),
    (
        "crates/biomcp-fetch/src/hosts_tests.rs",
        "Check credential-free exact publisher origins at the private admission boundary without a product inventory helper.",
    ),
    (
        "crates/biomcp-fetch/tests/transport/tls_builds.rs",
        "Count TLS configuration builds across hosted calls at the private constructor boundary without exposing test controls.",
    ),
    (
        "crates/biomcp-fetch/tests/grpc_worker/shutdown.rs",
        "Prove cancellation joins active network work at the private worker boundary without exporting shutdown control.",
    ),
    (
        "crates/biomcp-fetch/tests/quotas/expiry.rs",
        "Check idle bucket expiry after scaled refill without sleeping or exposing transport state.",
    ),
    (
        "crates/biomcp/src/grammar/reference.rs",
        "Generate and compare documentation at the private parser boundary without adding a public command.",
    ),
    (
        "crates/biomcp/src/mcp/worker_tests.rs",
        "Test worker panic completion and exact serialized tool result character boundaries that public requests cannot intentionally produce.",
    ),
    (
        "crates/biomcp/src/mcp/http/call.rs",
        "Prove HTTP failure recovery, safe diagnostic fields and hosted worker capacity and shutdown ownership at private boundaries.",
    ),
    (
        "crates/biomcp/src/config/credential_reads_tests.rs",
        "Observe forbidden credential environment reads at the private settings boundary while executing real commands.",
    ),
    (
        "crates/biomcp-adapters/tests/deferred_keys_tests.rs",
        "Exercise deferred credential admission through the public AlphaGenome adapter without reading binary settings.",
    ),
    (
        "crates/biomcp-types/tests/author_contracts.rs",
        "Exercise checked source-qualified author records and standalone native paper and work-group contracts before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/cell_line_contracts.rs",
        "Exercise checked cell identities, native variation members and provenance, and standalone consumer \
            contracts through the public library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/gwas_contracts.rs",
        "Exercise checked GWAS identities, exact probabilities and standalone consumer contracts through the public \
        library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/phenotype_contracts.rs",
        "Exercise neutral phenotype records and checked standalone source and section contracts before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/pathway_contracts.rs",
        "Exercise neutral pathway identities and checked standalone consumer contracts through the public library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/protein_contracts.rs",
        "Exercise neutral protein records and checked standalone consumer contracts through the public library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/study_contracts.rs",
        "Exercise exact study identities, neutral source metadata and bounded standalone consumer contracts \
            through the public library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/diagnostic_contracts.rs",
        "Exercise neutral diagnostic records, native identities and checked standalone consumer contracts \
            through the public library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/variant_coordinates.rs",
        "Exercise VCF coordinate parser boundaries and assembly-qualified full-reference validation strings through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/variant_values.rs",
        "Exercise copied common variant values and constructor boundaries through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/article_ids.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/disease_contracts.rs",
        "Exercise neutral disease records and standalone consumer contracts through the public library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/pgx_contracts.rs",
        "Exercise checked pharmacogenomics requests and neutral native record contracts before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/adverse_event_contracts.rs",
        "Exercise neutral report records and checked source-specific adverse event contracts before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/drug_contracts.rs",
        "Exercise published drug identifier, neutral record and consumer schema contracts through the public library API before adapters exist.",
    ),
    (
        "crates/biomcp-types/tests/core.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/gene_ids.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/gene_model.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/rust_api.rs",
        "Exercise the external Rust API consumer compile/runtime contract.",
    ),
    (
        "crates/biomcp-types/tests/biodata_switch.rs",
        "Exercise reviewed shared identifier identity, selector wire text and refusal codes through the consumer API.",
    ),
    (
        "crates/biomcp-types/tests/query.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/response.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/schemas.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/schemas/disease.rs",
        "Exercise standalone disease option metadata and synthetic envelope schema contracts before command registration.",
    ),
    (
        "crates/biomcp-types/tests/schemas/compatibility_cases.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/schemas/policy.rs",
        "Test reviewed schema pattern changes at exact paths and reject near misses.",
    ),
    (
        "crates/biomcp-types/tests/schemas/compatibility_recursive_cases.rs",
        "Exercise recursive public schema graphs and their compatibility constraints.",
    ),
    (
        "crates/biomcp-types/tests/schemas/temporal.rs",
        "Exercise native timestamp schema spelling and temporal property consistency.",
    ),
    (
        "crates/biomcp-types/tests/schemas/drug.rs",
        "Exercise public drug header and card disclosure boundaries through their serialized values and schemas.",
    ),
    (
        "crates/biomcp-types/tests/schemas/registry.rs",
        "Register public value and research query schema contracts for schema drift checks.",
    ),
    (
        "crates/biomcp-types/tests/schemas/ownership.rs",
        "Check published schema namespaces against the crate that defines each Rust type.",
    ),
    (
        "crates/biomcp-types/tests/schemas/values.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-types/tests/value_conventions/sweep.rs",
        "Exercise public text value construction, parsing, JSON and hashing through the Rust consumer boundary.",
    ),
    (
        "crates/biomcp-types/tests/value_conventions.rs",
        "Exercise checked values and schema contracts through the public library API.",
    ),
    (
        "crates/biomcp-adapters/src/monarch/native_contracts.rs",
        "Exercise retained gene-keyed Monarch rows through the private mapper without adding a gene-keyed command.",
    ),
    (
        "crates/biomcp-adapters/tests/alphagenome_registration.rs",
        "Exercise restricted source registration and required-key admission before a product section exists.",
    ),
    (
        "crates/biomcp-adapters/tests/batch_bounds.rs",
        "Exercise literal indexed lookup inputs outside command validation and empty/oversized batches or chunks beyond command limits.",
    ),
    (
        "crates/biomcp-adapters/tests/ctgov_refusals.rs",
        "Inspect registry refusal facts, legacy snapshot defaults and native continuation fields omitted from command JSON.",
    ),
    (
        "crates/biomcp-adapters/tests/custom_refusals.rs",
        "Inspect adapter refusal facts and legacy snapshot codecs omitted from command JSON.",
    ),
    (
        "crates/biomcp-adapters/tests/custom_refusals/collections.rs",
        "Inspect retained native member refusal facts, snapshot round trips and consumed counts omitted from command JSON.",
    ),
    (
        "crates/biomcp-adapters/tests/custom_refusals/stored.rs",
        "Inspect stored native relation refusal facts and source member indexes omitted from command JSON.",
    ),
    (
        "crates/biomcp-adapters/tests/custom_refusals/transport.rs",
        "Inspect native page consumption and retained member refusal facts omitted from command JSON.",
    ),
    (
        "crates/biomcp-adapters/tests/protein_storage.rs",
        "Read legacy serialized adapter values through the public storage type without a source request.",
    ),
    (
        "crates/biomcp-adapters/tests/pmc_body.rs",
        "Reconstruct complete source rows and retain JATS reference IDs and links omitted from command JSON.",
    ),
    (
        "crates/biomcp-adapters/tests/pmc_limits.rs",
        "Reject invalid table spans and duplicate table IDs before allocation with a tiny decoder boundary table.",
    ),
    (
        "crates/biomcp-fetch/src/cache/contention.rs",
        "Order independent SQLite snapshots and writer admission at the private transaction boundary without product hooks.",
    ),
    (
        "crates/biomcp-fetch/src/destination_tests.rs",
        "Cover public DNS address and redirect rejection tables without contacting private destinations.",
    ),
    (
        "crates/biomcp-fetch/src/grpc/dns_tests.rs",
        "Prove a stalled platform lookup cannot outlive its caller budget or begin a stream, without changing the host resolver configuration.",
    ),
    (
        "crates/biomcp-fetch/src/retry_tests.rs",
        "Cover Retry-After forms and backoff boundaries without one recording per table row.",
    ),
    (
        "crates/biomcp-fetch/src/xml_tests.rs",
        "Cover inline XML markup and decoder limits with many tiny inputs.",
    ),
    (
        "crates/biomcp-adapters/tests/alphagenome.rs",
        "Exercise the selected request encoding and complete bounded envelopes through the public protocol API.",
    ),
    (
        "crates/biomcp-fetch/tests/alphagenome.rs",
        "Exercise required-key routing through a synthetic stream.",
    ),
    (
        "crates/biomcp-fetch/tests/alphagenome/routing.rs",
        "Exercise required-key admission and placeholder credential routing through a local TLS stream.",
    ),
    (
        "crates/biomcp-adapters/tests/score_variant.rs",
        "Exercise the ruled scoring request and complete scorer admission through the public protocol API.",
    ),
    (
        "crates/biomcp-fetch/tests/batch.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/bundle.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/bundle/download.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/bundle/native.rs",
        "Exercise complete native TSV replacement sets and decoded bounds through the public transport API.",
    ),
    (
        "crates/biomcp-fetch/tests/bundle/rejections.rs",
        "Exercise rejected source rows, persisted revalidation and old-store migration through the public transport API.",
    ),
    (
        "crates/biomcp-fetch/tests/bundle/rows.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/cache.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/cache/credentials.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/cache/freshness.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/cache/storage.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/grpc_persistence.rs",
        "Exercise checked RPC cache admission and bounded recording replay through the public transport API.",
    ),
    (
        "crates/biomcp-fetch/tests/grpc_persistence/policy.rs",
        "Exercise credential containment, concurrent stream capture and atomic publication through the public transport API.",
    ),
    (
        "crates/biomcp-fetch/tests/grpc_persistence/process.rs",
        "Exercise ordinary checked reply cache reuse from an isolated second process through the public transport API.",
    ),
    (
        "crates/biomcp-fetch/tests/grpc_worker.rs",
        "Exercise blocking streaming RPC admission, local TLS HTTP/2, proxy routing, deadlines and cancellation through the public transport API.",
    ),
    (
        "crates/biomcp-fetch/tests/double_gzip.rs",
        "Exercise double-compressed replies and body limits on each layer through the public transport API.",
    ),
    (
        "crates/biomcp-fetch/tests/offline.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/paths.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/post.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/quotas.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/record.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/record/errors.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/record/metadata.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/retries.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/support/cache_process.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-adapters/tests/tensors.rs",
        "Exercise bounded tensor decoding with synthetic protobuf payloads through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/transport.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/transport/cancellation.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/transport/publisher.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/transport/network.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/transport/credentials.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-fetch/tests/xml.rs",
        "Exercise transport ownership, local server behavior and cache/record failures through its public API.",
    ),
    (
        "crates/biomcp-query/tests/article_search.rs",
        "Exercise concurrent pool publication/hydration, injected SQL corruption, external writers and API-only deadline boundaries.",
    ),
    (
        "crates/biomcp-query/src/store/privacy_tests.rs",
        "Prove caller ownership at the private query store security boundary before licensed routes publish continuations.",
    ),
    (
        "crates/biomcp-query/tests/gene_resolution.rs",
        "Mutate isolated cache freshness and preserve a local bundle failure under an API-only short command deadline.",
    ),
    (
        "crates/biomcp/tests/ratchets.rs",
        "Enforce repository structure and prove refusal of planted violations in isolated trees.",
    ),
    (
        "crates/biomcp/tests/ratchets/lints.rs",
        "Prove the shared Clippy settings reject planted scalar, banned-type and reason drift.",
    ),
    (
        "crates/biomcp/tests/ratchets/dependency_sources.rs",
        "Reject unapproved Git revisions and missing source policy inputs through the dependency guard.",
    ),
    (
        "crates/biomcp/tests/ratchets/proof.rs",
        "Enforce repository structure and prove refusal of planted violations in isolated trees.",
    ),
    (
        "crates/biomcp-query/tests/article_rights.rs",
        "Inspect raw replies, ranked pools and full-text refusal identities through public queries using synthetic transport.",
    ),
    (
        "crates/biomcp-query/tests/article_rights/authors.rs",
        "Prove private abstract refusal metadata survives raw cache reuse and rewrite legacy author continuation bytes through public queries.",
    ),
    (
        "crates/biomcp-query/tests/article_rights/contexts.rs",
        "Inspect retained citation reply bytes and distinguish absent contexts from rights refusals through public adapters.",
    ),
    (
        "crates/biomcp-query/tests/article_rights/lookups.rs",
        "Inspect stored replies and pools after failed or capped license lookups, and repeat reads, through public queries.",
    ),
    (
        "crates/biomcp-query/tests/article_rights/routes.rs",
        "Inspect synthetic transport replies for source-owned abstract and passage rights and bounded dependency admission.",
    ),
];

pub(super) fn check(root: &Path) -> Check {
    let crates = root.join("crates");
    if !crates.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(crates)? {
        let crate_root = entry?.path();
        for directory in ["src", "tests"] {
            for path in rust_files(&crate_root.join(directory))? {
                let relative = path
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/");
                let tokens = source::tokens(&fs::read_to_string(&path)?);
                let has_test = tokens.iter().enumerate().any(|(index, token)| {
                    token == "#"
                        && tokens.get(index + 1).is_some_and(|v| v == "[")
                        && tokens
                            .iter()
                            .skip(index + 2)
                            .take_while(|v| v.as_str() != "]")
                            .collect::<Vec<_>>()
                            .windows(2)
                            .any(|v| matches!(v, [left, right] if right.as_str() == "test" && matches!(left.as_str(), "," | ":")))
                        || tokens.get(index..index + 4).is_some_and(|v| v == ["#", "[", "test", "]"])
                });
                if has_test && !ALLOWED.iter().any(|(file, _)| *file == relative) {
                    return Err(format!("{relative}: inline test outside listed locations").into());
                }
            }
        }
    }
    let architecture = root.join("sdlc/planning/architecture.md");
    if architecture.exists() {
        documentation(&fs::read_to_string(architecture)?)?;
    }
    Ok(())
}

fn documentation(text: &str) -> Check {
    let section = text
        .split("### 10.4 ")
        .nth(1)
        .ok_or("missing test placement section")?
        .split("## 11.")
        .next()
        .ok_or("missing following architecture section")?;
    let listed: Vec<_> = section
        .lines()
        .filter(|line| line.starts_with("| `crates/"))
        .collect();
    let expected: Vec<_> = ALLOWED
        .iter()
        .map(|(path, reason)| format!("| `{path}` | {reason} |"))
        .collect();
    if listed != expected {
        return Err("architecture 10.4 differs from the reasoned test-file list".into());
    }
    Ok(())
}
