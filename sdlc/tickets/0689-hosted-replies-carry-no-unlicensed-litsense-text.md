# 0689 — Hosted replies carry no unlicensed LitSense text

Status: OPEN.

Milestone: 5

## Outcome

At hosted `restricted`, an article search with `source=litsense` keeps its PMID ranking and returns no `sentence` text. LitSense is labeled `per-record`. Any route that returns article text at `restricted` applies the per-record rule, including stored pools.

## Why

Row W1 of `records/2026-11-06-pre-release-review.md`: `litsense/mod.rs:60-70` returns body sentences with no license check, and recorded replies include noncommercial papers. That breaks Ian's ruling in `decisions/2026-11-04-hosted-source-modes.md`: "data elements ... we will not access them, share them, or return them back." Read in code, not run, because LitSense stays uncalled.

## Proof

- Proof: a recorded-reply hosted case at `restricted`, failing first, whose LitSense reply holds a sentence and asserts no sentence text reaches the caller, plus one for a stored pool (`article/page.rs:66-90`). Check whether PubTator returns passages the same way. No live LitSense call. Routine gate.

## Starts from

- Starts from: `crates/biomcp-adapters/src/litsense/mod.rs`, `disabled_use(Use::PerRecord)`, `pmc_oa/rights.rs`.

## Keeps

- Keeps: local and CLI use unchanged, and the ranking.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/src/litsense/`, `crates/biomcp-query/src/article/`, `tests/`.

## Amendment, 2026-11-06

Ian replaced the Outcome above, which dropped all sentence text: "if there's an article that solves a problem but we can't return the text due to the commercial limit, we have a way of doing that filtering in a safe way. We can still give the maximum return of papers to look at, but maybe that doesn't have the text." The hosted server is not touched for this, and there is no rush.

New Outcome: at `restricted`, every paper a search finds stays in the results, in rank order. Its text appears only when that paper's own license allows commercial use (`pmc_oa/rights.rs`). Otherwise the paper comes back as far as its rights allow, with a stated reason and where to read it. An unknown or unreadable license means no text. This holds on every route that returns article text, including LitSense, PubTator passages, full-text reads, stored pools and the source reply cache: withheld text is dropped in memory and never written to disk at `restricted`. A full-text read of a paper we may not return gives the paper's identity and reason, not a failure.

Proof adds: recorded replies with one permitted, one noncommercial and one unknown paper, showing three rows and one sentence; a store inspection showing no withheld text on disk; and a bound on license lookups, so a search stays within its deadline and fails toward no text. Still no live LitSense call.

PM ruling (m28714), under Ian's "You can keep on things that have no explicit non-commercial use stuff": the "unknown license means no text" rule covers body text: full text, body sentences and passages. Titles, abstracts and citation data follow their source's terms, so PubMed and Europe PMC abstracts stay when a record has no license. A record that states a noncommercial license loses its abstract at `restricted`, and the paper stays. The PMC OA allowlist is not applied to abstracts with no license. A LitSense sentence from a title or abstract follows this same rule.

PM ruling (m28745): `License::Tdm` withholds the abstract at `restricted`, the same as an explicit noncommercial license, and the paper stays. `planning/slices/article.md` already says text under a text-mining license is never served by a web mode.

PM ruling (m28760): a controlled one-time scrub, not a sanitized import. The existing store-clean path deletes withheld rows and rebuilds the file so old free pages and WAL hold none of those bytes. It runs on a copy, and the copy replaces the store only after hosting's file checks and study lookup pass. This ticket builds and proves the step on a synthetic store. Running it on hosting belongs to the deploy Ian approves. If hosting's article cache is separate from the snapshot, hosting may start that cache empty instead.

PM ruling (m28791): the scrub in the m28760 ruling is withdrawn. Hosting read its store: 0 reply rows and no article tables, only source bundles. Runtime replies live in each task's temporary copy, and a deploy starts fresh tasks from the image. So no scrub step and no store clean option ship. This ticket keeps the filter before anything is written, and every reply route.

From the ticket review, `records/2026-11-06-ticket-review.md`:

Before the final gate, make these changes and no others:

- Look up PubMed licenses before the source call, or release the source permit first, so a text check never waits on a permit its own call holds.
- Keep the withheld marker when a stored LitSense row is filtered again.
- Add the PubMed withheld marker only when it is absent.
- Write stored article rows only when the text changed.
- When a license lookup fails, say so with a reason distinct from a refusal, do not store the reply, and do not publish the pool as whole.

Stop widening Semantic Scholar stored routes. Keep what exists.

The per-ticket check script and Makefile goals do not land. Run the gate commands directly.

PM rulings on the review: when a license lookup fails or a paper is past the cap, the abstract stays, under the m28714 ruling; only body text is withheld. The bound of 20 records and 3 seconds is accepted, the reply states what went unchecked, and a timing job follows only if it hurts.
