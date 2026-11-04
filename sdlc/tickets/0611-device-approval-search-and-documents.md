# Device approvals are searchable, and approval documents are readable

Status: OPEN.
Milestone: 5

## Outcome

`search diagnostic` accepts `--source fda-pma` and `--source fda-510k` with name, manufacturer and date filters; `get diagnostic` for a PMA adds a `summary` section with the text of its Summary of Safety and Effectiveness Data; and `get drug ... approvals --source drugsfda` can return a submission's approval letter text.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 3. PMA and 510(k) records are readable only by exact number; `search diagnostic --source fda-pma --manufacturer "Foundation Medicine"` exits 2 with `usage.retired`; openFDA 510(k) search by device name finds K223435. There is no SSED text (for example `cdrh_docs/pdf17/P170019B.pdf`), and Drugs@FDA approvals give letter PDF links, not their text. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: `crates/biomcp-adapters/src/fda_pma/`, `fda_510k/`, `crates/biomcp-query/src/diagnostic/pma.rs` and `clearance.rs`, the retired-search mapping in the diagnostic query types, the PDF reader in `crates/biomcp-adapters/src/pdf/`, and the Drugs@FDA letter links.
- Rules: search uses openFDA's own device PMA and 510(k) endpoints, already called for get. Document text comes from FDA's own published PDFs: the existing bounded download (25 MiB) plus one pure-Rust PDF text parser, capped at 50 pages, no OCR or external tool (m24566); a scanned PDF without text says so. Text is the source's, never summarized. The `usage.retired` mapping for these two sources is removed.
- Proof: installed-binary cases: a manufacturer search and a device-name search on each source, P170019's SSED summary section, and one approval letter's text; an over-limit PDF gives the bounded reply.
- Changes: `crates/biomcp-adapters/src/fda_pma/**`, `crates/biomcp-adapters/src/fda_510k/**`, `crates/biomcp-query/src/diagnostic/**`, `crates/biomcp-types/src/query/diagnostic/**`, `crates/biomcp-types/schemas/diagnostic*.json`, `tests/cases/diagnostic/**`, `tests/replies/diagnostic/**`; diagnostic search sources, the PMA summary section, the letter text option, help, map, cases.
- Defers: nothing.
