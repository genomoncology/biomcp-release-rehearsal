# 0724 — Source text arrives clean

Status: OPEN.

Milestone: 5

## Outcome

Replies carry readable text: HTML entities decoded, damaged characters such as `?` for `≥` repaired at their source or marked, markdown escapes removed from trial results, FAERS codes shown as words beside the code, and GWAS p-values below the float range shown from the exact value, not 0.0.

## Why

Item 8: EMA `&nbsp;` and `?` for `≥`; trial results `\<` and `\^2`; FAERS outcome `6`; GWAS 0.0 below about 1e-300. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then recorded-reply cases for each. Check first whether EMA's `?` is in the source bytes or made by our decode; if it is in the source, say so and leave the value. One shared text cleaner, not one per adapter. Raw values stay available where they are identifiers.

## Starts from

- Starts from: `crates/biomcp-adapters/src/faers/detail_map.rs`, EMA, ClinicalTrials.gov and GWAS adapters.

## Keeps

- Keeps: meaning of source text. No guessed characters.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/`, `crates/biomcp-types/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Checked by reading data, not by running. EMA's `?` is in the source: a copy of the publisher's medicines file is valid UTF-8, has no replacement characters, and holds both `?10 g/dl` in 69 indications and real `≥` elsewhere. Leave the value, as the Proof says. A `?` cannot be spotted without guessing from text, so nothing is marked in replies. Add one line to `docs/cautions.md`.

Trial escapes such as `\<` are in the source's own values.

"One shared text cleaner" means one module with two named functions, entity decoding and backslash unescaping, called only for EMA and ClinicalTrials.gov text. It is not applied in `NativeText` or to every source, because backslashes and entities are not markup everywhere. It stays in `biomcp-adapters`, with no change to BioData.

All cleaning happens when a reply is read. Stored rows, word indexes and the bundle format do not change, so existing snapshots stay valid.

Trial result text deserializes straight into public types with no mapping step. Clean it in `ctgov/detail/results.rs`.

The Proof checks JSON for clean text. Markdown keeps the escaping that 0348 requires.

FAERS: reaction outcome, patient sex, drug role, action taken and reporter qualification get words beside the code, copied from openFDA's published field reference.

GWAS: `p_value` is left out only when it is zero and `exact_p_value` is present.

Land after 0718, which edits nearby EMA and FAERS files.
