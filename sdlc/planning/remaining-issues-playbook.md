# BioMCP remaining issues test playbook

Use this with the [November 10 findings register](../records/2026-11-10-product-review.md) and the owning ticket. The baseline is core `a84ee9721f7fb486755341286ae5512232ba9799` and hosting `a14558a99e408c1d89ac134cde75ecc9e1350063`; record the actual commit and executable hash for later runs. Every check below is **unrun in this review** unless its evidence explicitly names j36638. These are tests to perform, not claims that the program already fails them.

## Handoff to a testing agent

Read workspace/repo instructions, the owning ticket and its source paths. Use a clean worktree or pinned export. Do builds and tests through `pm job` on M5 or Yellow; M5 is the first choice because Yellow's November 10 compile failed at its configured missing sccache wrapper. Preserve that infrastructure failure rather than calling it a negative product test. Reuse a verified matching artifact where possible. The reviewed native macOS artifact and exact successful probe are in the findings register.

Give each run a ticket, hypothesis and stop condition. Pin Rust 1.95.0 and Python 3.13 where those tools are used. Isolate HOME, configuration and cache using the existing test helpers. Pass no ambient source keys or user state. Use synthetic recordings for malformed data, refusal, concurrency and authentication; live checks use only the existing public keyless source scope. Do not call KEGG or the licensed LitSense subset, spend on model calls, change real accounts, deploy or publish for these checks.

For each case return: source/artifact identity, host, initial cache/config state, exact command or MCP frames, exit/protocol result, relevant expected versus observed fields, request count and elapsed time where meaningful, and a path to bounded raw evidence. Label it confirmed, not reproduced, test setup failure or untested. Stop on a decisive witness and preserve it. A test passing once does not diagnose an earlier intermittent failure.

The existing case runner rejects an empty selection. List its actual names before choosing a narrow filter. Typical existing entry points, run through the owning `pm job`, are:

```sh
cargo test --locked -p biomcp --test cases -- --list
cargo test --locked -p biomcp --test cases gwas/
cargo test --locked -p biomcp-fetch --test cache
cargo test --locked -p biomcp-fetch --test offline
python3.13 scripts/test_nightly_live_check.py
make docs-check
```

Do not run every command above by default. Select the test that can decide the current hypothesis. New regression inputs belong in the existing fixture/case owners, not a parallel test framework. After a fix, review it and use one normal landing gate for a compatible batch.

## First priority regression cases

| Check | Setup and action | Required observation | Owner and boundary |
| --- | --- | --- | --- |
| P1. Unknown GWAS membership | Existing synthetic GWAS page with one valid qualifying association and one missing/invalid exact p-value; run threshold search. Repeat with all values incomparable and with uncertainty on an earlier page before a clean last page. | Never complete plus an exact qualifying total. Keep usable rows and diagnostics; true comparable zero/equality/underflow controls remain correct. Saved rereads retain uncertainty. | **Preserve uncertainty in GWAS threshold totals**; existing installed GWAS cases. |
| P2. Refused GWAS association | One refused association identity and one valid member; separately refuse a nested child only. | Exact total is withheld under the ticket's conservative mixed-refusal policy. Do not describe a rejected child as proof of a missing association. | Same GWAS owner, same fixture family. |
| P3. Incomplete query pool reuse | Seed a refused or stale protein observation, then issue the original query without a pool ID. Separately fail a later native page, recover it, and start a new search. | The incomplete pool is not selected as current. Explicit old ID still resolves under its original status/expiry rules. A failed uncached page can prove a new request; raw cached malformed replies need not bypass their TTL. | **Reuse only whole source snapshots**; query/installed tests. |
| P4. Concurrent pool updates | Hold two publishers/extensions at the existing transaction fixture; let a healthy newer current answer win, then complete an old partial/shorter write. Include identical-payload reuse and an old-format partial mapping. | A losing or old writer cannot remove/replace the newer current mapping. Stored payload and eligibility agree atomically. Old incomplete defaults are refused without a generic data purge. | Same shared-pool owner; one transaction-level witness plus representative public flow. |
| P5. Healthy lazy/local pools | Query a healthy protein page with continuation, then repeat; exercise study catalog paging with no upstream source outcomes. | Healthy reuse still works before the entire provider catalog is consumed; empty local source lists do not become failure. | Same owner; audit every caller's evidence wiring. |
| P6. Incomplete drug identity | Article and trial name expansion: one retained exact structure plus one refused possible match; then a capped candidate set whose total exceeds received rows. | No unique-identity assertion or expansion from that prefix. Existing fallback may succeed only with complete evidence; otherwise typed term plus explicit incomplete notice. Bounds and `--exact-term` remain effective. | **Require complete evidence for drug name expansion**; shared resolver plus both entry points. |
| P7. Resolver pooling and controls | Repeat a no-pool article search after unresolved expansion; seed an old implicit expanded pool. Include true no-match, two known structures and complete stereo/product rows of one structure. | Unresolved lookup cannot be made current merely because HTTP source states are OK. Old implicit unsafe expansion cannot bypass the fix; explicit pages remain stable. Correct unique/ambiguous/no-match controls remain distinct. | Same name-expansion ticket. |
| P8. Native trial consumption | Two source pages of two rows, total four; refuse one first-page row and provide the redundant full-final-page token. Repeat with an all-refused first page and unknown total. | First continuation offset two, admitted counts one then two, source order/token intact and no extra empty third page for known total. | **Advance trial pages by native record count**; installed trial cases. |
| P9. Revoked signing key | Controlled issuer publishes K1; authenticate, remove it, expire cache freshness, authenticate same kid again. | Refusal without unknown-kid trigger or restart. Retained K1 and normal K2 rotation pass controls; replacing material at the same kid is observed. | Hosting **Bound cached issuer signing key age**; real local auth routes with fake issuer. |
| P10. Issuer outage and load | At the freshness boundary, hold/fail one issuer fetch while concurrent known/unknown-kid calls arrive. Cancel one fetch; try recovery after the retry floor. | One coalesced refresh; fresh keys work only within freshness; stale outage gives 503, successfully removed key gives 401. Failed/cancelled attempts never reset successful age or bypass the attempt floor. Both verifier surfaces and config readers agree. | Same hosting owner; deterministic policy time plus event-ordered socket fixtures, no real issuer writes. |
| P11. Narrow cache reader | Cache a valid large reply, read with a smaller limit, then with the original wider limit. Controls: corrupt entry and oversized stale fallback. | Narrow read never consumes oversized bytes or deletes valid reusable state; wider read can hit cache. Corruption still evicts. | **Retain valid cache entry for narrow readers**; public Fetch API. |
| P12. Offline negative knowledge | Compare fresh 404, expired 404 and no cached entry in isolated offline mode. | Historical expired absence has a distinct truthful diagnostic; no network call and no stale current-absence success. | **Distinguish expired negative offline knowledge**; public Fetch API. |

The source and admission decisions, not clinical interpretation of a returned association, are what these fixtures prove. For biological examples, check exact provider fields and assembly/coordinate metadata; do not turn source statements into an independent medical conclusion.

## Usability and accuracy exploration

These additional tests are leads rather than new defect tickets. Start with the ones touching a changed owner; stop and report any decisive mismatch before broadening.

| Check | Procedure | What would be a defect or require a ticket |
| --- | --- | --- |
| U1. CLI/MCP parity | Start stdio, send initialize and wait, send initialized notification, tools/list and wait. Construct calls from the returned schema. Compare CLI JSON and tool structured data for one complete, absent, partial and failed synthetic query; then typed mode. Read replies before closing stdin. | Different identity/value/status/next command, duplicate payloads or wrong isError/error classification. j36638 proved help parity only. |
| U2. Grammar discovery | Use no args, help, missing required input, unknown option/boolean, explicit bad `--section`, and too-large limit. Test CLI and the advertised MCP schema. | Unclear or false recovery hint, ignored input, invalid input accepted, valid input rejected, or runtime work before a parse refusal. `get gene BRAF bogus` is two inputs by design, not an invalid-section test. |
| U3. Returned next commands | Follow returned pagination/section/disambiguation commands using the actual response, without hand-repair. Include empty first page, zero-match search, partial later page, changed source exclusions and expired pool. | Dropped selectors/build/threshold, loops, skipped/duplicate identities, stale pool silently switched, or continuation that cannot be executed. Preserve intentional explicit restart guidance. |
| U4. Identity and coordinates | Use existing recorded variant cases for rsID, CAID, HGVS and genomic form on GRCh37/38, including insertion/deletion boundaries. Compare normalized identity, assembly, reference/alternate and source coordinates. | Software mapping contradicts the declared source/coordinate contract. Different biological assertions across sources are not automatically a normalization bug. |
| U5. Failure versus absence | Synthetic 404, empty successful list, 429, 5xx, TLS refusal, malformed and oversized bodies, missing optional field and refused record. Include one failed source alongside one valid source. | Error relabelled absence/complete; discarded valid peer data; malformed/oversized conflated; false exact totals; next steps advertise unavailable behavior. |
| U6. Source policy and credentials | With fake source endpoints and two synthetic callers, vary exclusions and per-request keys, reuse warm raw/query caches, then change one caller's settings. Repeat through wrapper. | Cross-caller reuse of licensed/private data, a disabled source still called/returned, client-supplied policy/key headers trusted, or old key used beyond the documented cache bound. No real account mutation for this test. |
| U7. Cancellation and fairness | Hold a controlled upstream response, start two clients/users, cancel/timeout one and keep another running. Check actual socket close/worker completion and wrapper permits. | Work outlives its stated bound, abandoned work releases a permit while still consuming unbounded capacity, other user starves, or normal publication is cancelled by an unrelated caller. Use existing ownership/load fixtures; do not repeat the full prior stress collection. |
| U8. Budgets and health | A synthetic slow-but-within-budget provider plus one beyond-budget provider, with configured source/command caps and bounded concurrency. Compare health and an ordinary command. | Hidden three-second cap returns, request exceeds command budget, or a timing failure is labelled successful/absent. Live source health is an observation; inspect timing/cause before treating red as an outage. |
| U9. Local maintenance | Compare stdio and HTTP admission for sync, store clean, saves/replacement and fresh requests using an isolated store. | Parsed maintenance restriction differs from documented surface, unintended filesystem writes occur, or stored bundle/current-generation safety is lost. |
| U10. User docs and upgrade | Follow README to local commercial exclusions, key entry, proxy/CA help and first command. Check 0.9.3 migration mapping against current help. Use the prepared site's built links/install script at its actual revision. | Broken or misleading step, missing local instructions, mistaken hosted defaults, stale command names, or instructions that require a user to guess a section/flag. |

## Diagnose the live-source observation

The original scheduled run failed article search `AKT1 E17K` with ChEMBL/openFDA-label deadline errors and two Cellosaurus commands with unavailable. It already made a second attempt. The M5 artifact probe later succeeded, which establishes current usability but does not explain the earlier failures.

Start from the retained j36636/j36638 evidence and actual request construction. Cellosaurus uses `/search/cell-line` and `/cell-line/CVCL_0004`; unavailable covers both some HTTP and transport failures. A deadline can expire waiting for a source quota, transport or retry. Do not assign an outage or product root cause from the final reason alone.

After **Retain diagnostics for each live check attempt** lands, let the next relevant scheduled/targeted run capture the decisive details. If immediate diagnosis is needed, run each affected command once from the affected host in isolated keyless state, keeping the source envelope and timings; compare one exact public source request only if it distinguishes the suspected cause. Do not run the entire 24-command selection just to reproduce three. If the failure does not recur, record it as unresolved/not reproduced and retain the old failure. The existing final-source qualification owns this question.

## Diagnostics and documentation checks

For nightly diagnostics, the existing Python harness should supply first-fail/second-success, two different failures, malformed JSON and timeout. Verify both attempts survive with bounded output and unchanged final exit policy. Do not add raw source bodies, environment dumps or credential-derived values.

For policy docs, move the complete local instructions first. `scripts/check_docs.py` currently takes the first README JSON block as its commercial configuration. Update that invariant to a stable canonical example in configuration, retain its negative test, then run `make docs-check`. This is not a general Markdown link checker; separately follow the README anchor and use existing site checks when site inputs change.

## End the test assignment cleanly

Report actual counts and skipped cases. Keep raw output on Yellow with a manifest and verify the receiving copy. Preserve failed results alongside later successes. Let pm remove its job checkout and temporary folder; do not delete unrelated data or open work. File a new finding only when evidence establishes it, attach it to an existing ticket when it has the same owner/cause, and record uncertain cases with their next discriminating test.

Once accepted fixes settle, use the existing final qualification ticket for affected release sections and its three manual installed checks. No broad cold build, full mutation sample, all-platform rehearsal or AWS rollout is part of this exploratory playbook by default.
