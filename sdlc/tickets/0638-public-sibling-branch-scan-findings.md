# Secret-scan findings on public sibling branches are classified exactly

Status: COMPLETE.
Landed: 3310f95fdc9ebbcdf3c11e586466eee7175e332f
Milestone: 5

## Outcome

The production full-history secret scan passes with its full scope (`--all`), because the three `generic-api-key` matches in `src/entities/variant/resolution/coding_oracles/aliases.json` on the public repository's sibling branches are exact reviewed rows in `release/secrets.ignore.jsonl`, or, if any is a real credential, the append stops and the PM is told.

- Why: m25995. Scan j30304 of the public append for candidate 015aea0a failed on three matches in two blobs from commits 6c9c2d599 and 6d517ab20, which are ancestors of neither public main 57834215 nor the candidate; an ordinary full clone reaches them through public branches `biodata/biomcp-1.0`, `ticket/2041-merge-main-into-1-0-again` and `ticket/2098-m5-gate-host`. The 174-row ledger from 0633 has no rows for them.
- Starts from: 0633's rules and ledger, and j30304's retained commit, blob, line and rule receipt on Yellow.
- Rules: 0633's rules apply unchanged: one exact row per finding keyed by blob, commit, file, line and rule; each value reviewed locally; no value or hash written; no path, rule or scope exception; the scanner keeps `--all`. If `release/secrets.py` validates ledger commit bindings only against `rev-list HEAD` while scanning `--all` (m25996), make the binding check use the same reachable scope the scanner scans, and nothing wider. A real credential stops the work and is reported by path and line; history already public is not rewritten.
- Proof: the production scan of the public append exits 0; a test proves each new row matches exactly one finding and removing any one makes the scan fail.
- Changes: `release/secrets.ignore.jsonl`, `release/secrets.py`, `release/test_checks.py`; the rows, the binding scope if needed, and tests, including one where a row binds a commit outside the scanned scope and is rejected.
- Defers: nothing.
