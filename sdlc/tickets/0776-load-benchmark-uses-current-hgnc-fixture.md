# 0776 — Load benchmark uses the current HGNC fixture

Status: COMPLETE.
Landed: c2d1e45a5dde360ecc9cb9ad1626b364d9cc9c29

Milestone: 5

## Outcome

`make hosted-http-load` publishes at least 44,000 HGNC rows into an isolated cache before measuring, then measures 300 real HTTP calls and health checks while eight actual bundle publications run against that shared cache. Missing work still fails, and the existing budgets stay unchanged.

## Why

Final qualification j36135 completed its separate stress run but the load benchmark failed because its fixture published only 175 rows. The test must measure large shared-cache writes before it can qualify the hosted service.

## Proof

- Proof: j36135 is the failing old-code witness: 175 initial rows, 300 calls, eight reported syncs, no failed calls, and health responses all 200. The existing `make hosted-http-load` must pass after the setup fix, report at least 44,000 initial rows, retain 300 calls and eight syncs, and verify that the HGNC publication receipt changes across each sync. Run the previously skipped `make hosted-surface-check` after the load passes. Do not repeat the already completed stress suite.

## Starts from

- Starts from: `crates/biomcp/src/mcp/http/load_benchmark.rs`, the current `tests/replies/gene/main/hgnc` captures, `biomcp-fetch`'s strict TSV parser and replay publication path. The approved capture has 171 valid rows and a 53-column header; the withdrawn capture has five valid rows.

## Keeps

- Keeps: application behavior, strict TSV rejection, the 44,000-row threshold, 300-call workload, eight syncs, health checks, connection bounds and time budgets. Fetch's default replay cache is already isolated; no ambient-cache defect is claimed.

## Defers

- Defers: nothing within this benchmark fix. AWS deployment and publication remain outside this ticket.

## Changes

- Changes: the existing ignored benchmark's fixture mutation and writer setup. Trim only CR/LF so trailing tab cells survive. Use fresh replay Fetch importers sharing one explicit isolated ReplayCache, because one Fetch imports each replay bundle only once. Observe the existing `bundle_content` HGNC receipt through read-only cache access so cached metadata cannot masquerade as another publication. Require the initial large publication before starting measurements and retain all missing-work checks.

## Cause

`trim_end()` removed the last approved row's empty final TSV cell. Its template consequently had 52 columns instead of 53, so the parser rejected that original row and all 44,000 synthetic rows. The 170 surviving approved rows plus five withdrawn rows explain the observed 175 exactly. This is the only matching TSV-template helper in the scoped load/test setup. Separately, reusing the same Fetch let the later replay syncs return memoized bundle information instead of publishing again; the fresh writers and receipt check correct the benchmark without changing runtime code.

## Evidence

- Old-code job j36135: the stress section passed 62/62 checks, with 613 run receipts and no nonzero exit. The job then failed only at the 175-row load benchmark; the later hosted-surface goal did not run.
- Fixed load benchmark j36220 passed at `0e521b39a3ac61f8772d88aace9d9bf5ac478abe`: 44,176 initial rows, 300 successful calls, eight receipt-verified publications, no sync failures, and all 204 health samples HTTP 200. All calls occurred during publication; this run supplies no between-publication call-speed comparison. Health had 140 samples during writes and 64 between them. The observed 26.1-second measurement is a report, not a timing assertion.
- Hosted surface proof in j36223 passed both `mcp-hosted/served-surface-and-source-policy` and `mcp-parity/typed-hosted-catalog`, and formatting passed. That job then failed Clippy's `type_complexity` check on the setup return tuple; naming the tuple changed no benchmark behavior.
- Focused correction job j36228 passed at `7776c11403b3fff793d68be62998027828f1feee`: formatting, Clippy for the BioMCP binary and test targets with warnings denied, and the existing `repository_obeys_structural_contract` ratchet. Only comments, the setup type alias and these evidence references changed after the passing load measurement; no load or stress rerun was needed.
