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
