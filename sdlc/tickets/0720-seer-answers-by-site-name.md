# 0720 — SEER answers by site name

Status: COMPLETE.
Landed: a64ebb2f3fe3e14c15fb6f9ebd70e483e4185728

Milestone: 5

## Outcome

A caller names a cancer site in words and gets incidence or survival. Every missing required option is named in one error, period and stage codes carry labels, and a slow SEER call returns a bounded answer or a clear outage.

## Why

Item 4: no name search, so the tester paged through 73 sites to find Pancreas (40); required options appeared one error at a time; period codes were bare numbers; incidence and survival both timed out, while the web gives 13% five-year survival and 44% / 16% / 3% by stage. Report: `records/2026-11-06-life-sciences-test.md`.

## Proof

- Proof: Reproduce on current main first, because the test ran deployed revision 1bb97e64; if main already meets the Outcome, record the evidence and close. Fix the general cause and tune nothing to the report's examples. Then cases for a site by name, an ambiguous name, all missing options in one error, labeled codes, and a slow reply. One bounded live check finds out why both calls timed out, on host and in the deployed path. If the cause is the source's speed, the deadline and a stored answer are the fix to weigh.

## Starts from

- Starts from: `crates/biomcp-adapters/src/seer/`, `crates/biomcp-query/src/disease/seer.rs`.

## Keeps

- Keeps: SEER terms and pacing.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/src/seer/`, `crates/biomcp-query/src/disease/`, `tests/`.

## Amendment, 2026-11-06

The timeouts did not reproduce (m28873, m28896). The publisher answered in 0.2 to 0.3 seconds from the M5, and hosting's two deployed runs passed in about 10 and 2 seconds, on the deployed revision. The cause is unknown and hosting read no server logs, so no timeout fix is claimed. The usability work stands. Hosting also saw `value.unknown` problems on the membership and method fields; check those replies and fix them here.
