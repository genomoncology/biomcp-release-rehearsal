# 0721 — Variant recurrence accepts what it says it accepts

Status: COMPLETE.
Landed: e6d1549c30a75a26973019fd222b43c7ca122539

Milestone: 5

## Outcome

Every variant form the error message and schema list works in `variant_recurrence`, and a cohort mutation frequency comes back for a common variant. Forms it cannot use are not listed.

## Why

Mutation frequency row of the test: the tool rejected every variant form tried, including forms its own error lists. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then a case per listed form through CLI, MCP and the typed tool, and a test that the listed forms and the accepted forms come from one owner. Must work on hosting's existing cBioPortal snapshot.

## Starts from

- Starts from: `crates/biomcp-query/src/variant/recurrence.rs`, `crates/biomcp-query/src/variant/recurrence/matching.rs`, `crates/biomcp-types/src/query/recurrence.rs`.

## Keeps

- Keeps: 0710 study flags.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/variant/`, `crates/biomcp-types/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Checked against the public API. A common variant cannot finish today: reading a real KRAS cohort takes 17 calls at our cap of 100 rows, while the source returned all 1,670 rows in one 0.3-second request. The reply holds counts, not rows, so a larger page costs the caller nothing. Raise the default and maximum `limit` to 5,000, so a common variant reads every page in one call and reports `all_pages_read: true`. Paging stays for anything larger.

"Cohort mutation frequency" means the observed sample count beside the selected sample count once all pages are read. BioMCP computes no percentage, because the selected sample list is not always the set assayed for that gene.

The page step names Cancer Hotspots as its source. It must name cBioPortal, or the step disappears wherever Cancer Hotspots is off. Ticket 0693 holds the same one-word fix; make it here too.

Proof adds a multi-page cohort.
