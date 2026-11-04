# 0455 — Apply the bad-value rule in openFDA and NCI adapters

Status: COMPLETE.
Landed: b23339c4d64d57b802eeb6ecafca1d0d3155162e

Milestone: 5

Depends on: 0418, 0415, 0416

## Outcome

The openFDA family (`fda_510k`, `fda_pma`, `fda_cdx`, `maude`, `faers`, `drugsfda`, `openfda_label` and the recall adapters) and the NCI adapters follow the value rule. openFDA meta values are protocol values, and every other bad value is omitted with a problem.

- Why: split from 0418 so 0418 need not wait for the benchmark tickets that rewrite these adapters.
- Starts from: these adapter folders as 0412, 0413, 0415 and 0416 leave them, and the audit from 0418.
- Rules: the same rule and helpers as 0418. The shared openFDA envelope belongs to 0424. A property that leaves a required set takes a major under 0437's gate.
- Proof: one replay case per adapter edits one non-identity value and asserts record kept, field absent and problem named, and it fails on the base build; the audit has no pending site in these folders; the reply diff lists only those cases.
- Changes: the folders above, their cases, affected schemas and the baseline.
- Defers: the shared envelope (0424).
