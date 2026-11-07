# 0717 — Drug names expand across trial and article search

Status: COMPLETE.
Landed: ae70c51a9460833fbb78bab05a220f20f0e5287d

Milestone: 5

## Outcome

A trial or article search on a drug name also searches that drug's other names, so `sotorasib`, `AMG 510` and `Lumakras` give the same set. The reply lists the names it searched, and a caller can ask for the exact term only.

## Why

Item 1: `sotorasib` found 73 trials, `AMG 510` 40, `Lumakras` 16, all three together 75; `daraxonrasib` 91 papers and `RMC-6236` 102. The report calls this silent incompleteness, worse than an error for pharma. 0661 joined names across drug sources but not trial and article search. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then recorded-reply cases for a brand, generic and code name returning one set, with the searched names in the reply, and a name that resolves to nothing searching as typed. Names come from drug sources BioMCP already reads, as in 0661. Ambiguous names do not expand; the reply says so.

## Starts from

- Starts from: `crates/biomcp-query/src/drug/`, `crates/biomcp-query/src/trial/`, `crates/biomcp-query/src/article/`, 0661.

## Keeps

- Keeps: exact-term search, and paging over a stored pool.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-query/src/`, `crates/biomcp-types/`, `docs/`, `tests/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

Checked against real source replies. The Outcome stands, but synthetic cases alone cannot prove it. Real MyChem searches return several rows for one drug (`sotorasib` 3, `Lumakras` 2, `imatinib` 14, `metformin` 142): stereo forms of one structure and NDC product rows. Treating `total != 1` as ambiguous, and stopping at the first literal match, leaves those names unexpanded. `AMG 510` and `RMC-6236` match nothing, because the spaced form exists only inside one pipe-joined source value.

Changes to the build:

- Ambiguous means the candidates hold more than one structure key, not that a source returned more than one row. Product rows with no structure key, such as NDC rows, do not count. An ambiguous source, or one with no other names to give, passes to the next source and does not end the lookup. ChEMBL gave one record for `imatinib` and `metformin`, where MyChem has several structures. MyChem is the only source of `daraxonrasib`.
- Split source values that join several names, and compare names ignoring case, spaces and hyphens, only against candidates the source itself returned. This is identity comparison, not a word list.
- Cap the names in one search, configurable with a default. A capped search says how many names it left out. ClinicalTrials.gov returned 400 for a 110-name group in `query.term`.
- If the source rejects an expanded request, search the typed term and say so. Never report the caller's text as wrong.
- The lookup gets a bounded share of the time budget, so a slow lookup never costs the search.
- If every lookup fails, the reply says names were not expanded, and the result is not stored as a whole pool. A failed lookup is not the same as a name that resolves to nothing.
- Build no name quality filter. The reply lists the names searched, and `--exact-term` narrows them.

Proof adds fixtures in the real reply shape: several rows for one drug, several structures at one source, a pipe-joined value, and a code name differing only by a space or hyphen. It also adds a case where lookups use up their share of the time and the search still runs. A live check of brand, generic and code names runs on the built binary after landing, with drugs beyond the report's. A failure there is a new bug against landed work, not a landing condition.

PM ruling: this expansion is allowed by the amendment now on `decisions/2026-10-24-no-text-heuristics.md`.

## Amendment, 2026-11-06

PM ruling, from the review at the top of the hosting stack. These three are fixed before landing, on the stack's top branch, replacing the earlier allowance to land with a live failure:

1. ClinicalTrials.gov gets a size limit on the name group, because one long chemical name made it reject the whole group for sotorasib and imatinib. This is a provider limit, not a name quality filter. The notice counts names left out.
2. The name cap always keeps the record's preferred name, and providers that take one name per request use it first.
3. A reply says only the typed term was searched only when every grouped provider fell back.

## Amendment, 2026-11-06

PM ruling, second pass. A name group takes names only from records that describe this one drug. Leaving out combination product rows is a rule about records, not a name quality filter. When the walk cannot show one drug, the search uses the typed term and says so. `paracetamol` gave 64,226 trials against 3,395 for `acetaminophen`, from names like `Cold` and `HEB`, so this is fixed before landing. Official names on the drug card are a separate ticket.

## Amendment, 2026-11-06

PM ruling, replacing the single-ingredient wording above. Product rows give no names when the drug has rows of its own, and product-only hits give none either; a record with nothing but product rows still gives its names. Single-ingredient rows alone left 242 store brands such as `HEB` and `Pain Relief`. A brand stated only in a product row is searched only when typed. Some broad ChEMBL trade names stay, and that is accepted for 1.0.
