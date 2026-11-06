# Stances from the pre-release review

The PM adopted these on 2026-11-06 from the weighing in `records/2026-11-06-pre-release-review.md`. They settle calls that came up more than once.

1. A stored result is the current answer only when it is whole. Partial or stale results may be kept for their own pages but are never reused for a new search.
2. The rights check belongs at every route that returns text. Any route returning article text at `restricted` applies the per-record rule, whatever the source's label.
3. Judge hosted exposure on the real path. The core sits on loopback behind the wrapper. A finding that needs a raw socket or a spoofed header is fixed in docs or cheaply, not by hardening the core. A finding that reaches through the wrapper keeps its full weight.
4. Scanner findings are classified and fixed at their source. Scanner rules are not widened, and the classification goes on the ticket that reports the failure.
5. Nothing runs first in the release. A proof or stress test that only the release runs moves into the routine gate or ahead of the rehearsal. Publish commands stay no-ops until the tag, with read-only checks before it.
6. State a policy once. README and release text link to `docs/configuration.md` for source policy and do not restate it.
7. Captures from noncommercial or licensed sources in a public repo are synthetic. History stays as it is.
8. When a later ruling of Ian's overrides older decision text, the PM marks the older record at once.

The weighing's ninth call was Ian's: "if there are any cosmetic issues that we can get done, prioritize them and work on them. We have developer capacity, so get those done." Hygiene that no person meets is in scope. 0711 to 0716 carry it.
