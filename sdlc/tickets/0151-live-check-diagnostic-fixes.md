# 0151 — Fix the live-check diagnostics that 0068's first run exposed

Status: WITHDRAWN. Duplicate: 0069 (f7fa47ba) and 0070 (092c2794) already cover all three diagnostic outcomes

Milestone: 3

## Outcome

The live-check command reports drift that a reader can trust. It follows a skipped case's predecessor chain instead of dropping its followers, compares a live reply with its recording under the same cache and retry conditions, and records enough of each request plan and status to show why a case drifted.

- Why: 0068's first run, j9937, made 332 requests and flagged 24 drifts, but its review found three diagnostic defects: skipped predecessors silently skipped their followers, cache and retry differences showed up as drift, and request-plan and status changes lacked a witness. A scheduled run on these diagnostics would report noise.
- Starts from: the live-check command from 0055, 0069 and 0070; the j9937 evidence on the evidence store.
- Proof: offline cases built from the j9937 evidence, with no provider call: a follower of a skipped case is reported as skipped with its reason; a reply that differs only by cache or retry is not drift; a changed request plan or status carries its witness. Then make landing-gate runs through pm job. After this lands, 0068 hands the recipe to Jarvis for scheduling without repeating the whole run.
