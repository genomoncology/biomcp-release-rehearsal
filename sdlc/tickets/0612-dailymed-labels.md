# Drug labels found only on DailyMed are readable

Status: OPEN.
Milestone: 5

## Outcome

`get drug ... label` reads a label from DailyMed by SPL set ID when openFDA does not carry it, with the same sections as openFDA labels, and a label published only as a PDF returns its text through the bounded PDF reader.

- Why: `planning/eval-gap-recheck-2026-11-04.md` item 2: there is no DailyMed-only or PDF label source. DailyMed is the NLM's public label archive and carries labels openFDA lacks. Ian, 2026-11-04 (m24381): "All this stuff should get fixed in 1.0."
- Starts from: the openFDA label section after 0608, `crates/biomcp-adapters/src/openfda_label/` as the model, and `crates/biomcp-adapters/src/pdf/`.
- Rules: a new `dailymed` drug source declared with its published terms (public domain, NLM), its own adapter, quota and cases, never a fallback hidden inside openFDA. Sections are read from the SPL XML by LOINC section code into the same section names 0608 uses. Text is the source's.
- Proof: installed-binary cases: a label present only on DailyMed answers by set ID with two sections; a label on both gives the same section text from each; a PDF-only label gives text or a clear reason.
- Changes: `crates/biomcp-adapters/src/dailymed/**`, `crates/biomcp-adapters/src/lib.rs`, `crates/biomcp-query/src/drug/dailymed.rs`, `crates/biomcp-types/src/source.rs`, `tests/cases/drug/dailymed/**`; a DailyMed adapter and drug source, label section mapping, source registry, help, map, cases.
- Depends on: 0608
- Defers: nothing.
