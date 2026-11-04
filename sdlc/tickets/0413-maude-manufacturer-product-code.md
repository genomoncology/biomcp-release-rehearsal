# 0413 — Filter MAUDE device reports by manufacturer, product code and problem

Status: COMPLETE.
Landed: cf3f50426238c328576f18664212ced8cf4d67eb

Milestone: 5

## Outcome

`biomcp search adverse-event --source maude` accepts `--manufacturer` (phrase on `device.manufacturer_d_name`), `--product-code` (exact on `device.device_report_product_code`, repeatable and joined with OR) and `--problem` (phrase on native `product_problems`). The device phrase becomes optional when one of these is present. FAERS and VAERS still refuse these options. Help says one report can list several devices, so filters may match different devices in one report.

- Why: `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. MCP-Bench's test-kit step searches device reports by problem, and 0.4.6 filtered by manufacturer. Repeated product codes are the allowed form of 0.4.6's hand-made genomics code list, which the no-text-heuristics rule bans.
- Starts from: `crates/biomcp/src/retired/adverse_event.rs` (both flags are retired today), `crates/biomcp/tests/runner/adverse_event_retirements.rs`, `crates/biomcp-types/src/adverse_event/query.rs`, `crates/biomcp-query/src/adverse_event_maude/rows.rs` and the slice rows at `sdlc/planning/slices/adverse-event.md` 441 and 511, which this replaces.
- Rules: follow `sdlc/planning/architecture.md` and the shared source rules in `decisions/2026-10-28-benchmark-gaps-join-1-0.md`. NCI returns zero, not an error, for a wrong field name, so every NCI proof includes a positive case that a wrong build would fail. Retirement becomes source-aware, including when `--source` follows the flag. Making the phrase optional changes the adverse-event search schema and takes its major bump.
- Proof: replay cases for each filter, two product codes, a filter without a phrase, FAERS and VAERS refusals in both argument orders, and an unknown code; bounded captures.
- Changes: the files above, schemas, help, the slice rows, the upgrade table and cases.
- Defers: nothing.
