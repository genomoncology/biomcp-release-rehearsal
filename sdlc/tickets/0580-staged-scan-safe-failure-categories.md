# Staged artifact secret scan names its failure category

Status: OPEN.
Milestone: 5

## Outcome

When the staged-artifact secret scan cannot finish, it reports a fixed safe category (preparation, filesystem, validation or scanner), the stage and the artifact name, and never raw exception text or scanned content. A developer can then reproduce the public rehearsal's exact failure from the retained packages and fix its cause.

- Why: public rehearsal run 41447525003 at 06b491 passed the candidate and all five platform qualifications (Intel macOS in 101m40s, under the new 120-minute limit). Then the stage job's "Scan every staged publication artifact for secrets" exited 2 with only "Secret screening failed", with no path or rule findings, so attestation and every later job were skipped. `release/secrets.py` catches preparation, filesystem, validation and scanner errors into that one message. A real credential finding is not shown; the message cannot tell.
- Starts from: `release/secrets.py` (the staged scan and its catch-all), the stage job in `.github/workflows/release.yml`, the five native packages and raw logs on Yellow at `biomcp-redux-evidence/0294/public-rehearsal-41447525003`.
- Rules: map each caught error class to a fixed category and print the category, the stage and the artifact file name; keep raw exception text suppressed; keep every scanner, ledger and archive guard. Then reproduce the scan on the retained packages locally and fix the actual cause. Do not rebuild platforms to expose it, and do not waive or skip the scan.
- Proof: tests show each error class prints its category and no exception text; a local run on the retained packages fails before the cause fix and passes after; the next public rehearsal reaches attestation.
- Changes: `release/secrets.py` and its tests, plus whatever the cause fix names.
- Defers: nothing.
