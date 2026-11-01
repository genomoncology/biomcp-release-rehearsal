# 0555 — Bind reviewed history exceptions to the published history

Status: COMPLETE.
Landed: de098f5275124620f0125e947d9447e196b17d4c

Milestone: 5

## Outcome

Every row in the reviewed secret-scan exception ledger names a commit reachable from the candidate revision alone, and the history scan checks reachability against that revision instead of every local ref. A local branch can no longer hide a ledger row that the published history lacks.

- Why: the private no-op release rehearsal (run 41345375200, m19177) failed at the candidate history scan with `history-ignore-unreachable-commit` on ledger row 115, which names e59d2b857f4f3012577e74f4d36dc5cedb70f7e7, the reviewed ClinicalTrials.gov pagination cursor fixture, outside main's ancestry. Local scans passed because `release/secrets.py:271-272` computes reachability with `rev-list --all`, which includes auxiliary refs. The rehearsal upload carried only main and the tag, as the public append will.
- Starts from: `release/secrets.py:255-285`, `release/secrets.ignore.jsonl`, `release/test_security.py:283` and the rehearsal evidence.
- Rules: reachability uses only the candidate revision being scanned, never `--all`. Check every ledger row once against main's history. Rebind a row only when the same file and blob are reachable from main and the reviewed reason still holds, by changing its commit to the main-reachable commit that introduced that blob. Leave rows whose blob and path main never contains unchanged, because the scanner skips them and the public append may bring their history; the coordinator's inventory (m19181) found three rows to rebind (115, 117, 119) and 29 to leave. Add no new ignores and no broader rule, file or blob matches. Push no auxiliary or private history.
- Proof: an outside-in test builds a repository where an exception's commit sits only on a side branch and shows the scan of main fails with `history-ignore-unreachable-commit` even though the side branch exists locally; a scan of current main from a clone holding only main passes; the existing security tests pass; the rehearsal rerun passes the history scan.
- Changes: the secret scan helper, the exception ledger and their tests.
- Defers: none.
