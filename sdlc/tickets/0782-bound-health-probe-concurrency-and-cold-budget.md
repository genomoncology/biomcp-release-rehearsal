# 0782 — Honor configured health probe budgets

Status: COMPLETE.
Landed: d807e4d8a473fd9ebdc45fda525eb7ae793a6672

Milestone: 5

## Outcome

Health probes honor the shared command deadline and the ordinary configured per-source deadline. The health-only three-second cap no longer overrides both settings. Genuine refusals, HTTP failures, malformed replies and exhausted deadlines remain failures with truthful diagnostics.

## Why

Ian reproduced varying failed sources twice with roughly sixty uncached probes sharing a three-second per-source budget; the reported November 8 nightly GWAS failure needs the same policy checked.

## Proof

- Through the owning health surface, show that a valid recorded response delayed beyond the former three-second cap succeeds within the configured command/source budgets. Show the regression before the fix and use deterministic virtual delay rather than a quiet-machine timing assertion. A real HTTP error and exhausted command or source deadlines must still fail truthfully.
- Read the actual nightly witness before treating it as the same cause. A bounded live sample is optional only if needed to settle the policy; do not flood all sources repeatedly.

## Starts from

- `crates/biomcp-query/src/health.rs::run` passes `deadline.child(PROBE_BUDGET)` with a hard three-second cap. `Fetch::leg` already applies the ordinary configured source cap. Use that existing path and shared deadline, retaining `command_timeout_seconds` as the command setting.

## Keeps

- Per-source truth, real error exit status, cache bypass for live probes, existing fan-out and source quotas, and a bounded whole command. Reuse existing configuration; do not silently turn timeouts into success, add blanket retries, or serialize sources so queued probes expire before starting.

## Defers

- No new monitoring service, retry framework or source-availability claim based on fixtures.

## Changes

- Health probe deadline selection and its owning fixtures/documentation.
