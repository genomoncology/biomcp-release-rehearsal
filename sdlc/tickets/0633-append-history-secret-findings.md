# Every secret-scan finding in the appended history is classified exactly

Status: COMPLETE.
Landed: 5fac5bf8fa4f0e3b343987df2e63aebfbdddc222
Milestone: 5

## Outcome

The production secret scan of the merged history (public BioMCP main plus the approved candidate ed9c36a96) passes because each of its 50 `generic-api-key` findings is either an exact reviewed row in `release/secrets.ignore.jsonl` or, if any is a real credential, the append stops and the PM is told.

- Why: m25323. Scratch merge 38c1a2a1c (parents: public main 57834215d, then ed9c36a96, the exact candidate tree; 77 old tags including v0.9.3 resolve) gives 50 findings in full-history scan j29778 against the unchanged 124-row ledger. 42 align with earlier reviewed candidates (35 public identifiers, 4 controlled fixtures, 2 non-credentials, 1 documented example) but are not yet exact rows for this ancestry; 8 are new: `tests/article_asset_terminal.rs` (2 blobs), `sdlc/records/2146-refuse-an-empty-supplement-download-build.md` (2 blobs), and the golden cases `search-trial-melanoma-recruiting.json:8`, `ctgov-studies-melanoma-recruiting-limit3.json:168`, `melanoma-default-55b650b18997.json:289` and `dbsnp-refsnp-113488022-a20abb0708c2.json:1`. The append needs Ian's approval, and he must not be asked with the gate red.
- Starts from: `release/secrets.ignore.jsonl` (rows keyed by blob, commit, file, line and rule, each with a reason), the scan in `release/secrets.py`, and j29778's retained commit, blob, line and rule receipt.
- Rules:
  - One row per finding, keyed exactly by blob, commit, file, line and rule as the existing rows are; no path, rule or pattern exceptions.
  - Each of the 8 new findings is reviewed by looking at the value locally; the reason says what it is (for example a public ClinicalTrials.gov page token). No value, value hash or partial value is written anywhere.
  - The 42 aligned findings become exact rows only after confirming the blob is the same content earlier reviewed.
  - A finding that is a real credential stops the work: no row, report to the PM by path and line only. History in public main is already public, so the owner rotates it; the team does not rewrite public history.
  - Findings in the public 0.9 history and in the candidate are both covered, since the production gate scans the merged history.
- Proof: the production scan of the scratch merge exits 0 with the updated ledger; a test proves each new row matches exactly one finding; removing any one row makes the scan fail.
- Changes: `release/secrets.ignore.jsonl`, `release/test_checks.py`; the ledger rows and their test.
- Defers: nothing.
