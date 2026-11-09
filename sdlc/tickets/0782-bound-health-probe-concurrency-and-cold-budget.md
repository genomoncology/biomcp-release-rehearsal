# 0782 — bound-health-probe-concurrency-and-cold-budget

Status: OPEN.

Milestone: 5

## Outcome

Cold health probes have a bounded, realistic request budget and concurrency policy, so ordinary source latency and the command's own fan-out do not produce misleading red checks. Genuine refusals, HTTP failures, malformed replies and exhausted deadlines remain failures with truthful diagnostics.

## Why

Ian reproduced varying failed sources twice with roughly sixty uncached probes sharing a three-second per-source budget; the reported November 8 nightly GWAS failure needs the same policy checked.

## Proof

- Reproduce the current policy through the owning health surface with controlled cold responses and an observable in-flight counter; show the regression before changing it. Verify the chosen concurrency bound, a valid delayed response, a real error and a genuinely exhausted deadline. Do not make a fast success requirement depend on a quiet machine.
- Read the actual nightly witness before treating it as the same cause. A bounded live sample is optional only if needed to settle the policy; do not flood all sources repeatedly.

## Starts from

- The existing health command/probe scheduling, source request budgets, cache-bypass policy and health fixtures on main 99e6da6b. Confirm exact code and existing configuration before choosing the smallest fix.

## Keeps

- Per-source truth, real error exit status, cache bypass for live probes, source-specific quotas and a bounded whole command. Choose measured defaults and reuse existing configuration or expose the policy as configurable; do not silently turn timeouts into success or serially multiply an unbounded wait across all sources.

## Defers

- No new monitoring service, retry framework or source-availability claim based on fixtures.

## Changes

- Health probe policy and its owning fixtures/help documentation as required by the chosen configuration.
