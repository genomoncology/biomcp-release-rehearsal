# 0791 — Handle owned refresh cancellation in the hosted date fixture

Status: COMPLETE.
Landed: 22cff14e4845844247726fefe3c0909f085a7ca5

Milestone: 5

## Outcome

The installed CLI trial `gene-bundle/hosted-past-refresh-states-date` verifies the hosted copy date and CLI stale status while permitting the server to cancel its owned background refresh on shutdown. Unexpected foreground request or fixture failures still fail the trial.

## Why

The final native release rehearsal at runtime source 0ad failed this trial on Mac ARM64 and both Linux architectures after 0781 correctly made shutdown interrupt background downloads.

## Proof

- Proof: use the existing installed CLI trial and a deterministic witness for interruption during refresh; demonstrate the old fixture failure and the repaired assertions without retrying for a green result. Run the owning slice and normal landing gate. Reuse recorded qualification evidence for unchanged production inputs.

## Starts from

- Starts from: main 41956b257e5aa8625fa114e35c4dee78cf7091e7; `crates/biomcp/tests/runner/bundle_pinned.rs`, existing network stub and publication peer helpers. Rehearsal 42725440933 jobs 129530855778, 129530855870 and 129530855832 each report 740 passes and this one failure, with CONNECT EOF or TLS Broken pipe on background request ordinal 2. Confirm this diagnosis before changing the fixture.

## Keeps

- Keeps: prompt owned-worker shutdown; caller cancellation independence; single-flight refresh; exact hosted date/content/status assertions; strict fixture validation for unrelated requests. Do not globally swallow network errors or weaken the shutdown assertions from 0789.

## Defers

- Defers: no runtime redesign, new test framework or unrelated test cleanup. If diagnosis shows a runtime defect, report its witness before expanding this ticket.

## Changes

- Changes: the existing hosted bundle date trial and the narrow existing fixture support it requires.
