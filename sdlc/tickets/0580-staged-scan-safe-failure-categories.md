# Staged artifact secret scan names its failure category

Status: COMPLETE.
Landed: a68f0cfb819d6316b7824ea2292927ecdc642aba
Milestone: 5

## Outcome

When the staged-artifact secret scan cannot finish, it reports a fixed safe category (preparation, filesystem, validation or scanner), the stage and the artifact name, and never raw exception text or scanned content. A developer can then reproduce the public rehearsal's exact failure from the retained packages and fix its cause.

- Why: public rehearsal run 41447525003 at 06b491 passed the candidate and all five platform qualifications (Intel macOS in 101m40s, under the new 120-minute limit). Then the stage job's "Scan every staged publication artifact for secrets" exited 2 with only "Secret screening failed", with no path or rule findings, so attestation and every later job were skipped. `release/secrets.py` catches preparation, filesystem, validation and scanner errors into that one message. A real credential finding is not shown; the message cannot tell.
- Starts from: `release/secrets.py` (the staged scan and its catch-all), the stage job in `.github/workflows/release.yml`, the five native packages and raw logs on Yellow at `biomcp-redux-evidence/0294/public-rehearsal-41447525003`.
- Rules: map each caught error class to a fixed category and print the category, the stage and the artifact file name; keep raw exception text suppressed; keep every scanner, ledger and archive guard. Then reproduce the scan on the retained packages locally and fix the actual cause. Do not rebuild platforms to expose it, and do not waive or skip the scan. PM ruling 2026-11-02 (m20472): the cause is 10 private-key pattern matches per architecture in the runtime image's Debian libgnutls30, each byte-equal to a public self-test fixture in GnuTLS 3.7.9 `lib/crypto-selftests-pk.c`. The scanner may treat a finding as reviewed only when its bytes equal one of those named public fixtures, recorded with the upstream file, tag and a digest of the public fixture. Every other finding, including a new key in the same library, path or rule, stays fatal. No library, path or rule exclusion. A later GnuTLS version with changed fixtures fails until reviewed again.
- Proof: tests show each error class prints its category and no exception text; a test injects a fresh key into a copy of the library and the scan fails; the retained packages scan clean with exactly those fixtures recognized; a local run on the retained packages fails before the cause fix and passes after; the next public rehearsal reaches attestation.
- Changes: `release/secrets.py` and its tests, plus whatever the cause fix names.
- Defers: nothing.
