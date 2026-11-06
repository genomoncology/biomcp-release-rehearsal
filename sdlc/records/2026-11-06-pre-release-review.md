# Pre-release whole-project review of 2026-11-06

Ticket 0688. Three fresh reviewers read main f33f42a55 read-only; a fresh agent weighed their findings against main b886c33ec. The PM's rulings follow the weighing except where noted in `decisions/2026-11-06-review-stances.md`. Rows W1 to W57 are the weighing's merged findings; tickets cite them by row.


Weighed 2026-11-06 by a fresh agent, against current main b886c33ec, which differs from the reviewed f33f42a55 only by the ticket file 0688. Findings in reports A, B and C were merged into 57 rows. "New ticket" names are proposals; they are grouped in the section after the table.

## Summary

- 57 merged findings: 30 accept, 13 accept a cheaper fix, 4 defer, 9 reject, 1 for Ian.
- Claims the reviewers got wrong or missed, from records they did not read:
  - The 12 staged-artifact scanner findings **were classified** (m26534, 2026-11-05): one harmless compiled text pairing of the public AlphaGenome address with a neighbouring message, repeated in 12 package forms, no credential. The fix, ticket 0665 "Use the canonical AlphaGenome HTTPS endpoint", **landed** at 5018220c7 and is on main. Reviewers B and C read 0293's progress line, which is stale and still says "unclassified".
  - 0640's branch already lists stage, staged scan, staged Homebrew, plugin and attestation sections. C's correction to it is already done there.
  - `HOMEBREW_TAP_TOKEN` exists on the public repository (name read with `gh`, value not read).
  - The hosting repo answers Reviewer A's "questions for hosting": the wrapper caps work per signed-in user, the core listens only on loopback behind it, the disk is 20 GiB and temporary, and the wrapper logs tool, status and duration per call.
  - The BioData pin recheck is recorded (m20413).
  - The blind comparison gate was dropped by Ian (m23337, m23350).
  - The cache does prune itself: every reply write deletes rows 30 days past freshness (`cache.rs:291`).
- One thing the reviewers missed: `licensed` is **not** a usable disable group. `registry/groups.rs` lists only `open` and `noncommercial`, so the retired `restricted` alias cannot simply be swapped for `licensed` in the README (W3).
- Land now, no new work: "Markdown help lists every flag" (0686) and "Ignore ledger binds to landed main" (0687) are both accepted and ready for landing.
- New work is about 6 to 8 developer days in small tickets. None is a redesign. Hosted-affecting tickets go first, to match Ian's order (AWS before 1.0). All of it can land on main under the deploy freeze.

## Measurements

Both ran under key 0688 on m5, with no key, against openFDA only. KEGG and LitSense were not called.

- j32050, b886c33ec, exit 0: past the end, MAUDE, drug recalls, device recalls, 510(k) and label searches all answer 404 "No matches found", with no total. FAERS answers 400 past offset 25,000 ("Skip value must 25000 or less").
- j32056, b886c33ec, exit 0: Drugs@FDA, name METFORMIN HYDROCHLORIDE: exact total 190, word total 212. At offset 191 the exact search gives 404 and the word search gives 200 with rows and total 212. IBUPROFEN: 267 and 272, same result. For names where the two totals match (LIPITOR, ACETAMINOPHEN, AMOXICILLIN, OXYCODONE HYDROCHLORIDE) both give 404, which then reads as total 0.
- From existing records: j32033 and j32021 built and ran the tool proof in 268 s and 280 s total under load, an upper limit on what W6 adds to a gate whose median is 1,079 s. I did not separate the 30 to 60 s Reviewer C quotes.

## Findings

Holds: Yes, Partly, No. Stance files are in `sdlc/decisions/` unless noted. "Every fix" means `2026-10-27-1-0-ships-every-known-fix.md` and `2026-11-05-everything-in-1-0.md`.

| ID | From | Label | Holds? | Stance | Cost | Gain | Measurement | Recommendation and ticket |
|---|---|---|---|---|---|---|---|---|
| W1 | B1 | LitSense returns sentences from noncommercial papers to hosted callers | Yes. `litsense/mod.rs:60-70` returns text with no license check. Recorded sections are RESULTS, DISCUSS, METHODS and INTRO, so this is body text, not abstracts. Narrower than B says: default article source is PubMed, so it needs an explicit `source=litsense`, and neither docs nor skill offer it. The typed schema does. | `2026-11-04-hosted-source-modes.md`: "data elements ... we will not access them, share them, or return them back." The stance answers B's question 1, so the code fix needs no ruling. 0585 labelled LitSense `open` "(licensed subset still uncalled)". | About 10 lines plus one recorded case. Hosted loses sentence previews and keeps the ranking. Per-paper license lookup rejected: one more request per paper on a 1 per second source. | Stops a breach of Ian's direct ruling on the live service. Reached only when an agent picks LitSense. | None; LitSense not called | **Accept, before release.** New ticket "Hosted replies carry no unlicensed LitSense text". Test stored pools too. Live mitigation is Q1. |
| W2 | A1 | Partial article search stored and reused for 24 hours | Yes. `article/mod.rs:75-78,130`; hosted strips `--fresh` (`hosted.rs:138`). Copies in `edges.rs`, `recommend.rs`. | `2026-10-23-the-cache-is-a-local-sqlite-database.md` (pool is a stored snapshot) covers stable paging, not a failure as the answer to new searches. | A few lines in three places, one case. More upstream calls during an outage; quotas bound them. | One bad minute at a source stops shaping a day of answers for every hosted caller, who has no refresh. | Reproduced by A (j32005) | **Accept, before release.** New ticket "Article pools publish only whole answers", with W19. |
| W3 | C4 | README commercial config turns off PubMed, ClinicalTrials.gov, ClinVar | Yes (j32009), and the alias lists them (`registry/legacy.rs`). | `2026-11-01-no-backwards-compatibility.md` bars the alias. Ian 2026-11-03 (m22678): "There shouldn't be any limits to anything that's free for commercial use." This overrides the older line in `2026-10-24-sources-are-open-by-default.md` and `2026-10-26-new-sources...`, so C's licensing question is already answered. | Half a day: delete alias (about 10 sites), add `Use::Licensed` to `groups::USES` (without it `licensed` is refused as an unknown source and COSMIC and OMIM components have no switch), README case, notes line 40. | Every commercial user following the README gets failures on the opening examples. Common path. | Reproduced by C | **Accept, before release.** New ticket "Commercial config matches the three-use ruling", with W4. README gives `["noncommercial", "licensed"]`. PM marks the two older decisions as superseded on this line. |
| W4 | B4, C10, A17 | README hosted paragraph is the retired policy | Yes. `README.md:65` against `hosted_use.rs:27`. | `2026-11-04-hosted-source-modes.md` | An hour | Operators get true guidance | Read and probed | **Accept.** Same ticket as W3. Link to `docs/configuration.md`. |
| W5 | C5, B6 | Main's secrets check is red | Yes (j31992, j32003). 0293 does not depend on 0687 (checked). | `2026-11-05-release-finds-no-bugs.md` | 0687 accepted, ready. Add to 0293's dependencies. Cause: rows bind to a branch commit that squash landing discards. Binding to path and blob, or rewriting at landing: under a day. | Secret history scan works at all | Records read | **Accept.** Land 0687 now. Add a follow-up to it, since j30917, j31795 and j31992 repeat the failure. |
| W6 | C6 | Installed proof is red on main; the routine gate never runs it | Yes. `Cargo.toml:49-51`. 0686's Make goal runs one proof and is not in `landing-gate`. | `2026-11-05-release-finds-no-bugs.md` | Land 0686. Add `--proofs` with `BIOMCP_TEST_PYTHON`: a compile and at most about 4.5 minutes per gate. | Release-only failures show at landing | j32033, j32021 as upper limit | **Accept.** Land 0686 now. New small ticket "Tool proofs run in the routine gate". After 0686, rerun the installed gate to settle C's doubt about a second cause. |
| W7 | C1, B6 note | No green full release run; 12 scanner findings unowned | Partly. Classified (m26534), fixed (0665 at 5018220c7). No green run since is true, but that is the hold in `2026-11-05-hosting-validates-before-1-0.md`, not a defect. C's fix by binary offset would break 0665's "Keeps" (never weaken the scanner). | `2026-11-05-hosting-validates-before-1-0.md` | Nothing new. Correct 0293's stale line. | Clean record for the next reviewer | Mail and `git merge-base` | **Accept the cheaper fix:** correct the record. The staged scan passes under 0640 when the hold lifts. No scanner exception. |
| W8 | C2 | Publishing settings have no owner or preflight | Yes. No `release-publication` environment (read with `gh`). The token exists. PyPI's trusted publisher cannot be read from `gh`. | `2026-11-01-no-manual-release-pause.md` names the environment; `CLAUDE.md` requires Ian for third-party changes; `sdlc/plan.md` already says Ian approves these settings. | Half a day for the check, plus Ian's minutes. | A failure at PyPI after GitHub and GHCR are public leaves a half-published 1.0, and PyPI versions cannot be reused. | `gh` read-only | **Accept.** Owner and read-only check named in 0294. Account change is Q3. |
| W9 | C3 | Intel Mac qualified 16 seconds inside its 120-minute limit | Yes. 119m44s; `release.yml:96`, `recover-section.yml:34`. | None | Minutes, free on a public repository. C's better option (full suite once per platform) saves an hour but 1.0 has no deadline. | No wasted tag run; more cases have landed since | Hosted record | **Accept the cheaper fix:** both limits to 180. New ticket "Public release text and limits". |
| W10 | B2 | Terms text and use labels disagree | Partly. B says the evidence is not in reach. It is: m22678 and Ian's `notes/hosted-biomcp-academic-sources-and-terms.md` give verbatim terms: eRepo and Allele Registry CC0, cBioPortal ODbL "unless otherwise noted", Hotspots ODbL, Mutalyzer and VariantValidator silent on results. So the registry text is mostly stale. Real gap: cBioPortal says some studies restrict commercial use, and nothing here reads that. | `2026-11-04-hosted-source-modes.md`; 0585 | Hours: rewrite six terms strings and `unverified: true`; Hotspots to `attribution`. B's word-matching test is dropped as more than this needs. | Users stop seeing "unverified" on sources we reviewed. Ian's ruling is met for restricted studies. | None | **Accept the cheaper fix** in the W3 ticket. cBioPortal studies are Q2. PubMed abstracts stay as labelled under the same review: no question needed. |
| W11 | B3 | Real captures from noncommercial sources in the public repo | Yes for WHO PQ (136 KB) and Semantic Scholar (about 21 real files). B is wrong that nothing covers captures: `2026-10-26-new-sources-follow-open-by-default.md` allowed capturing Semantic Scholar. But it does not cover publishing them. `AGENTS.md:5`, 0476's precedent and the terms in Ian's notes do. | `AGENTS.md:5`; `instructions/decisions/2026-10-19-public-repos-skip-history-scrubs.md` means history stays. | Most of a day: synthetic rows, case updates. New check skipped until it happens a third time. | Keeps barred data out of an MIT tree going to the main public repo. Harm is legal, not user-facing. | Counts from B | **Accept the cheaper fix, before the append.** New ticket "Noncommercial captures become synthetic", on 0294. No history rewrite. PM rules this; it is a one-line note in Ian's approval packet, not a question. |
| W12 | A2 | Good trial page reported as failed when a side call fails | Yes. `trial.rs:49-54` sets partial, `:93` overwrites. | None; every fix | A small change, one case | 10 good trials stop vanishing. `/version` is cached for a day, so the window is narrow. | Reproduced by A | **Accept.** New ticket "Status and totals tell the truth". |
| W13 | A3 | Paging past the end of an openFDA search gives a looser search or a false zero | Yes, now on the live source. | None | One shared helper, seven adapters, one extra request on an exhausted page | No false "0, complete" and no silent word-match. Needs a caller offset past the total; agents do set them. | **j32050, j32056** | **Accept.** Same ticket. Also return a clear refusal for FAERS past 25,000, not the raw 400. |
| W14 | A4 | Protein cards cut sections to five rows with no sign | Yes by reading. `protein/sections.rs:33-52`. | `2026-10-24-dense-facts-first-with-a-token-budget.md` allows short defaults, not silent ones. No-compatibility stance allows the schema change. | Small, plus schema. Overlaps 0680 "Typed follow-ups are typed calls". | Reader can tell five from five of thirty | None | **Accept.** Same ticket, after 0680. |
| W15 | A5 | OpenTargets drug paging counts accepted rows | Yes by reading. `drug/native.rs:126` against `native_window.rs:40`. | None | One line, one test | No repeats or early stops. Rare. | None | **Accept.** Same ticket. |
| W16 | A6 | Label search by application number keeps a wrong total | Yes by reading. Whether the source returns such rows is unknown. | None | Small | Rows and totals agree. Possibly rare. | None | **Accept.** Same ticket. |
| W17 | A7 | HTTP server exits on one accept or thread failure, with no message | Yes. `http.rs:132,142,168`, and `:112` discards the error. On hosting the file limit case is unlikely and a restart comes from ECS, but all calls on that task drop with no reason logged. | None; Ian's priority is a solid AWS service. | About ten lines, one forced-failure test | Service sheds load instead of stopping | Reproduced by A, harsh setup | **Accept, first.** New ticket "Hosted server keeps serving and says why", with W18, W22, W23. |
| W18 | A8, B5 | Idle sockets make `/health` and calls return 503 | Yes on a raw port (reproduced twice). Not reachable on our hosting: ALB and wrapper hold client sockets, and the core on loopback gets whole requests from the wrapper. The docs claim is false. | None | One line (readers up to the existing worker bound) plus doc correction | True docs. Self-hosters without a front end get more margin. | Reproduced by A, B | **Accept the cheaper fix** in that ticket. No separate reader pool. |
| W19 | A9 | Stored pool plus a newly disabled source gives Failed | Yes (j32005). Also seen statically in m22975, never ticketed. Cannot occur on our hosting while the ceiling stays `restricted`. | `2026-11-01-no-backwards-compatibility.md`: no migration | Small: disabled set in the pool key, a little more storage | A working local search stops failing with no next step | Reproduced by A | **Accept.** With W2. Keep the failing branch for explicit pool IDs. |
| W20 | A10 | One caller can use all hosted capacity | No for hosting. The wrapper caps work per verified user (`docs/security.md:16`, `docs/limits.md:67`). Shared keyless pacing is designed. | 0548 "Defers: per-user caps"; hosting docs | None | None left in core | Read | **Reject.** Hosting's load test owns the numbers. |
| W21 | A11 | Hosted cache has no disk ceiling | Partly. Rows do prune at 30 days past freshness; A missed this. No size limit, but the disk is 20 GiB, temporary, and fresh at each task replacement. By my arithmetic about 150,000 distinct 100 KB replies per task per month fill it. | `2026-10-26-measured-entity-caps-need-no-ask.md` wants a measured cap | No core code now. A disk alarm is hosting's. | Covers a long-running task | None; no traffic numbers yet | **Accept the cheaper fix:** ask hosting to watch disk use. Add a size limit only if a load run shows growth. |
| W22 | A12 | Hosted server tells operators almost nothing | Partly. Wrapper logs user, tool, status and duration (`src/proxy/logging.rs:44`). It cannot see partial or failed sources inside a 200 reply, or why the core stopped. | `CLAUDE.md`: no secrets logged. Hosting keeps query text out of logs. | About a day: one line per call with tool, duration, status, failed source names, no arguments | Lets hosted validation see W2 and the partial rate. Needed before validation is worth much. | None | **Accept the cheaper fix** in the W17 ticket. Add A's damaged-table `/health` test. |
| W23 | A13 | Hosted loses partial answers at the time limit | Likely, by reading. Fails safely. | Every fix | One setting, a replay test first | Fast sources survive a slow one | None | **Accept** in the W17 ticket if the test fails. |
| W24 | A14 | Large cache write can stall reads and `/health` | Unproved risk | None | Measured in the load run | Unknown until measured | None | **Defer to 0641** "Real-time and multi-process tests". It owns this. |
| W25 | A15 | Moved cache cannot resync after an interrupted sync | Yes. `bundle/store.rs:50-66`, reproduced. | Every fix | A few lines, one test | Rare local failure | Reproduced by A | **Accept.** New ticket "Status and totals tell the truth", or any store lane. |
| W26 | A16 | Hot-path performance | Risk. No `[profile.release]` confirmed. No budget broken. | `2026-10-23-least-code-easy-upkeep-fast.md` | Unknown until measured | Unknown | None | **Defer past release:** pm idea, trigger: hosted load run shows TLS setup or store open dominating a call. |
| W27 | A18 | Seven small adapter items | Likely. I spot-checked the pathway step (always Reactome) and CPIC's missing `order`. | Every fix | Under a day in all | Small | None | **Accept**, failing test first; drop any item with no failing test. Same ticket as W12. |
| W28 | C7 | Load tests ordered after the final rehearsal | Yes. 0641 and 0670 wait on 0293. | `2026-11-05-release-finds-no-bugs.md` | Ticket edits | Races are found before the candidate, not after | `pm daily` | **Accept.** Reverse the dependencies; 0641 also takes j31824. Same fix for 0676 "Hosted schemas hide fields" and 0649. |
| W29 | C8 | Publication code first runs at v1.0.0 | Yes, and the no-op rehearsal is by decision. | `2026-11-01-no-manual-release-pause.md`, `2026-11-02-public-rehearsals.md` | Minutes for `twine check`; an hour for the installer byte check. | Catches a bad wheel before an unrepeatable upload | None | **Accept the cheaper fix.** Name the unproved commands in 0293. No TestPyPI or scratch registry unless Ian wants it. |
| W30 | C9 | Plugin users get a raw `{{front-map}}` | Yes by trace: `skills/SKILL.md:6`, copied raw at `release/workflow.py:119-126`. | None | Hours | Every plugin user's skill guide | None | **Accept.** New ticket "Plugin skill ships expanded". Assert it in 0640's plugin section, which also settles whether `./skills/` is found. |
| W31 | C11 | Public users see the switch before the release exists | Yes. 0294 rules the order, and "one operator session" narrows the window but does not close it. A failed tag run leaves main switched. | `2026-10-27-append-history-to-biomcp.md` allows a tag on the merge commit. The ruleset (non-fast-forward, no bypass) allows the later main update. | Ticket text only | A failed tag leaves public main untouched | `gh` read-only | **Accept.** Amend 0294: push the tag first, fast-forward main after publication passes. |
| W32 | C12 | Two owner holds exist only in text | Partly. Blind comparison: Ian moved it to evals and said to drop the dependency (m23337, m23350). `2026-10-28-blind-cli-comparison-before-publishing.md` lacks a marker. Hosting hold already chains through 0640 to 0293 to 0294. | Those mails; `2026-11-05-hosting-validates-before-1-0.md` | Two lines | Clean record | Mail | **Accept the cheaper fix:** mark the decision as handed to evals, name the hold in 0294. Do not add a gate. |
| W33 | C13 | biomcp.org stays on 0.9 after 1.0 ships | Yes. 0401 lands after 0294 from a branch dated 2026-10-28. | No stance. Ian's approval of the release covers the docs that ship with it. | Half a day to a day to refresh the branch before the cut | 1.0 users do not land on 0.9 docs | None | **Accept the cheaper fix:** refresh the branch before the cut and land it in the release session. No pipeline step. PM can rule; little release time. |
| W34 | C14 | Public release assets carry internal wording | Yes. `release/workflow.py:138,187`; `release/notes.py:22`. | `CLAUDE.md` public docs rule; no-compatibility stance says notes list every change | An hour | Clean public text; adds the macOS floor | Reproduced by C | **Accept.** New ticket "Public release text and limits". |
| W35 | C15, A note | Nightly live check failed 11 times and hides why | Yes. Partly on drift: classification alone fails only on failure, uncertain or outage, but `result != 0` also counts. | `CLAUDE.md`: don't lose work; no stance on nightly | An hour to print counts and source names | A check people stop reading protects nothing | Job list | **Accept the cheaper fix.** Print the reasons first, then read the evidence for j31978 before deciding on anything more. Any real failure gets a ticket. |
| W36 | C16 | Gate's Python is not pinned | Yes. 3.12 in `release/toolchain.json`, `python3.13` in 36 files, m5 default 3.9.6. | None | One line, plus settling which version is true | Fewer false failures: j31827, j30853, j32002 | None | **Accept.** With W6. |
| W37 | C17 | Gate runs only on macOS arm64 | True, and accepted. Linux runs as a 0640 section; Windows as a hosted section. | `2026-11-05-release-finds-no-bugs.md`; `instructions/decisions/2026-10-17-github-actions-only-for-public-releases.md` | None | None | None | **Reject**, stance. |
| W38 | C18 | Stale docs and plan text | Yes, spot-checked. | Every fix | One pass, under half a day; skip `release/qualification.md`, 0640 rewrites it | Docs tell one story | None | **Accept.** PM owns the sdlc parts; docs parts in the W3 ticket. |
| W39 | C19 | Per-ticket proof sediment | Lead, not proof | `2026-10-23-least-code-easy-upkeep-fast.md` | A sweep | Upkeep only | None | **Defer past release:** pm idea, trigger: 1.0 published. |
| W40 | C20 | Gate failures do not explain themselves | Yes. `check_front_map.py:86` bare assert. | None | Minutes; put the caps in 0679 "Typed tool descriptions stand alone" | Five to six gates lost time | None | **Accept.** With W6. |
| W41 | C21 | `cargo doc` fails on links | Yes. Not in the gate; crates are not published, so no reader. | None | Under an hour | None a person meets | None | **Reject:** nobody reads it. |
| W42 | C21, B11 | Build paths in the binary | Yes. No private names found. | No promise of byte-reproducible builds | One flag | None a person meets | None | **Reject:** no harm found. |
| W43 | C21 | Wheel has two copies of the binary | Yes. Second copy looks deliberate: it is what `uvx biomcp-cli` runs (`server.json:8`). Costs about 16 MB compressed. | `2026-11-01-no-backwards-compatibility.md` does not reach it; the name is the current package | A launcher, plus new test work | Smaller download | None | **Defer past release:** pm idea, trigger: a size complaint or PyPI limit. |
| W44 | C21 | 0293 names 44 dependencies | Yes. `pm lint` reports it. | pm lint limit | Minutes | Clean lint | `pm lint` | **Accept**, with W28. |
| W45 | B7 | `--hosted` with no config allows `academic` | True, by design | 0619; `infra/lib/config.ts:238` refuses anything else on our hosting | None | None | None | **Reject**, stance. |
| W46 | B8 | Hosted stdio ignores `hosted_max_use` | True in code, and it is what Ian ruled. | `2026-11-04-hosted-source-modes.md`: "local CLI and stdio keep every configured source." | One doc sentence | Clear scope | None | **Accept the cheaper fix:** doc only, W3 ticket. Reject the code change. |
| W47 | B9 | Any header value earns its own pacing bucket | True, but our wrapper strips client key headers and README tells operators to. | 0505 | None | None on our hosting | None | **Reject**, stance. |
| W48 | B10 | Tag push is the only publication control | Yes. Only ruleset is "Main Branch Protection" (branch, no tags). | `2026-11-01-no-manual-release-pause.md` bars a pause, not a rule on who can tag. | One account setting | Low: only org admins can tag now | `gh` read-only | **Ian**, Q3. |
| W49 | B11 | AlphaGenome Apache-2.0 protos have no notice | Yes. Proto headers are Apache; `release/notices.py` and license checks do not name them. | `AGENTS.md:5` | Minutes | License duty met | None | **Accept.** Ticket "Public release text and limits". |
| W50 | B11 | `BIOMCP_SQLITE_DIAGNOSTICS` undocumented | Yes. Proof-only setting (m-record on 0566) in the shipped binary. | None | One doc line | Small | None | **Accept the cheaper fix:** one line in `docs/configuration.md`. |
| W51 | B11 | No warning about patient identifiers in query text | Yes for public docs. Hosting's packet covers hosted. | `CLAUDE.md` Focus | One line | Cheap care for clinical users | None | **Accept**, W3 ticket. |
| W52 | B11 | NCI key tier is Fallback | True in code. Our hosting sets no server NCI key (no config or infra match); the wrapper sends the user's own. | `2026-10-24-keyed-sources-run-only-with-the-user-s-key.md` and 0503 | None | None on our hosting | Read | **Reject** for now. Reopen if hosting adds a server key. |
| W53 | B11 | `docs/cautions.md` says CPIC is noncommercial | Yes, line 54 only. Release notes are correct; B misread them. | m22678 (CPIC is CC0), `cpic/mod.rs:14` | One line | True docs | Read | **Accept**, W3 ticket. |
| W54 | B11 | Apt pins may vanish | Risk. Fails closed at rebuild, before anything is published. | None | Dated archive plumbing | Low | None | **Reject:** fails safely, fix when it happens. |
| W55 | B11 | BioData pin recheck not recorded | No. m20413: crates tree, `Cargo.toml` and `Cargo.lock` byte-identical to 08d9399. | `2026-11-02-biodata-publication-approved.md` | None | None | Mail | **Reject**, wrong. |
| W56 | B11 | Plain `serve --http` skips the call cap | Yes. `http.rs:349-362`. Docs state the cap with no limit. | None | One doc line | True docs | Read | **Accept**, W3 ticket. |
| W57 | C tickets | Ticket corrections | Mostly. Exceptions: 0640's section list (done on its branch), 0640's Windows VM drop (leave it to 0640), blind comparison (W32). | Various | Ticket edits | Accurate tickets | Checked 0293, 0294, 0640, 0641, 0060, 0401 | **Accept.** Withdraw 0060 "PM count checks": all dates are past and it holds milestone 1 open. Other corrections as C lists. |

## Proposed tickets, in order

1. Land 0686 and 0687. Correct 0293's stale line. Reverse the 0641, 0670 and 0676 dependencies. (W5, W6, W7, W28, W44)
2. "Hosted replies carry no unlicensed LitSense text". (W1)
3. "Hosted server keeps serving and says why". (W17, W18, W22, W23)
4. "Article pools publish only whole answers". (W2, W19)
5. "Commercial config matches the three-use ruling". (W3, W4, W10, W38 docs, W46, W50, W51, W53, W56)
6. "Status and totals tell the truth". (W12 to W16, W25, W27)
7. "Tool proofs run in the routine gate". (W6, W36, W40)
8. "Noncommercial captures become synthetic". (W11)
9. "Plugin skill ships expanded". (W30)
10. "Public release text and limits". (W9, W29, W34, W49)
11. 0294 edits. (W8, W31, W32) 0401 edit. (W33) Nightly reasons. (W35)

## Questions for Ian

### Q1. Should LitSense come out of the live hosted service now?

The code fix (W1) goes ahead regardless, because your 2026-11-04 ruling already covers it. The question is the service running today, since you froze deploys on revision 4 while Amazon tests. Today an agent that asks for LitSense gets sentences from papers that may not allow commercial use. It is not the default source.

- **A. Turn LitSense off by a hosting config change now.** Cost: one deploy during Amazon's testing, and agents lose an opt-in source until the fix ships. Hosting must confirm it can be a config-only change.
- **B. Leave it until your next approved deploy, then ship the core fix.** Cost: no deploy, but the gap stays open for as long as the freeze lasts.

Recommendation: A, because you said you don't want to invite any legal issues, and this is the one finding where a rule of yours is being broken on the live service. B is reasonable if the Amazon testing window is short.

Ian answered 2026-11-06: neither urgency applies. The server is not touched, because current use is connectivity testing. The long-term fix keeps every paper and withholds only text the paper's license does not allow (0689 amendment).

### Q2. What should happen with cBioPortal studies that restrict commercial use?

cBioPortal's terms say data is open "unless otherwise noted", and that some studies restrict commercial use in their study information. BioMCP does not read that, and hosting bakes in the study list.

- **A. Keep cBioPortal on and accept the gap.** Cost: hours for the wording. Risk stays.
- **B. Review the study list once and exclude flagged studies in `restricted`.** Cost: about a day, plus redoing it when the list changes. Someone must read the study notes; agents would flag candidates and not judge the terms.
- **C. Turn cBioPortal off in `restricted`.** Cost: hosted users lose study search and recurrence.

Recommendation: B. It meets your ruling and keeps the source. Mutalyzer and VariantValidator terms say nothing on results; I recommend they stay on with no further review.

Ian answered 2026-11-06: B, as a study-level flag in `restricted` (0710). Sources with no explicit noncommercial terms stay on.

### Q3. Publishing account settings, at the final approval

No hurry, since 1.0 has no deadline. These change outside accounts, so they need you.

- The `release-publication` environment and PyPI trusted publisher are needed, or PyPI fails after GitHub and the container are already public. Approve with the final packet: minutes of your time. Waiting past that is not an option for the release.
- A tag rule limiting who can create `v*` tags on the public repository. Cost: one setting. Gain: low, since only admins can tag now. Options: add it, or skip it.

Recommendation: approve both in the final packet.

## Recurring calls, written as stances

The PM may adopt these except the last, which is Ian's.

1. **A stored result is the current answer only when it is whole.** Partial or stale results may be kept for their own pages but are never reused for a new search. (W2, W19)
2. **The rights check belongs at every route that returns text.** Any route returning article text at `restricted` applies the per-record rule, whatever the source's label. (W1, W10)
3. **Judge hosted exposure on the real path.** The core sits on loopback behind the wrapper. A finding that needs a raw socket or a spoofed header is fixed in docs, or cheaply, not by hardening the core. A finding that reaches through the wrapper keeps its full weight. (W18, W20, W45, W47, W52)
4. **Scanner findings are classified and fixed at their source.** Scanner rules are not widened, and the classification goes on the ticket that reports the failure. (W5, W7)
5. **Nothing runs first in the release.** A proof or stress test that only the release runs moves into the routine gate or ahead of the rehearsal. Publish commands stay no-ops until the tag, with read-only checks before it. (W6, W8, W28, W29)
6. **State a policy once.** README and release text link to `docs/configuration.md` for source policy and do not restate it. (W3, W4, W38, W53)
7. **Captures from noncommercial or licensed sources in a public repo are synthetic.** History stays as it is. (W11)
8. **When a later ruling of Ian's overrides older decision text, the PM marks the older record at once.** Three stale records misled this review: the 2026-10-24 commercial config line, the 2026-10-28 blind comparison, and 0293's progress line. (W3, W7, W32)
9. **For Ian: does "every known fix" include hygiene that no person meets?** W41, W42 and W54 are rejected on my judgment, not a stance, because nobody is harmed. Your 2026-10-27 words say "everything we know about", and your 2026-11-05 words put AWS first. If you want them in, each costs under an hour except W54.

Ian answered 2026-11-06: do them. W41, W42, W54 and the deferred W26, W39 and W43 became 0711 to 0716. W20, W37, W45, W47, W52 and W55 stay rejected: each is by design or was wrong, so there is nothing to fix.

## Checks not made

- No live hosted server, no wrapper run, and no LitSense or KEGG calls. W1's live share of noncommercial papers is unmeasured.
- Findings marked "by reading" have no failing test yet; each ticket starts with one.
- Yellow evidence for the nightly and the stage log were not read.
- W21's fill estimate is arithmetic with an assumed reply size.
- No clinical or legal judgment of any source's terms; I report what the recorded terms say.

# Report A: correctness, stored state, reliability, operability, performance

## BioMCP pre-1.0 adversarial review: correctness, stored state, reliability, operability, performance

### Snapshot

- Commit: main `f33f42a55e040fccfc3394b516799f4baaf89133`, 2026-11-05 22:09 -0500, "Record landed ignore-ledger repair". Rust 1.95.0.
- Read from a pinned export, not the shared checkout. All paths below are relative to the repository root.
- Scope: one of three reviewers. Security, data rights, dependencies, release, gates, tests, docs and tickets belong to the other two, so the dependency table, gate time by stage and ticket judgments are theirs. I mention only what touched my area.
- Checks ran through ticket 0688 on m5. Job j32005 passed and holds every probe result below. Job j31996 stopped on a mistake in my own probe script, so it gives no evidence.
- Probes used a debug build on macOS with recorded replies. No live provider was called.

### What the system must never do

BioMCP answers medical research questions from public sources and marks what it could not reach. The promise that matters most here: a caller must be able to tell a full answer from a partial one, and one source's bad minute must not change the answer for long or for other people. Most findings below break that promise, or take the shared service down for everyone.

### Verdict

**Ready after the listed fixes.** I found no corrupt data and no lost stored data. Every partial answer I ran was labelled partial. The blockers are one stored-result defect that spreads a passing failure to every hosted caller for a day, one defect that hides a good trial page, a few paths where rows or totals go wrong under a clean status, and two ways a small problem takes the hosted service down or marks it unhealthy. All are small fixes, and none needs a redesign.

Timing below follows the stance "everything found goes into 1.0" (2026-11-05) and "1.0 ships every known fix" (2026-10-27). Where I say after release, I give the reason, and the weigher should check it against those stances.

### Before release

| # | Finding | Why it blocks | Smallest fix |
|---|---|---|---|
| 1 | A partial article search is stored and reused for 24 hours | One source's brief failure gives every hosted caller the short answer for a day, and hosted callers have no way to refresh it | Do not point `pool_current` at a pool when any source failed |
| 2 | A good trial page is reported as failed when the `/version` or count call fails | Ten good trials are hidden and the exit is 4, against the code's own stated intent of partial | Stop `search_for` overriding the partial status |
| 3 | Paging past the end of an openFDA search gives a looser search (Drugs@FDA) or "total 0, complete" (the rest) | A word-match answer or a false zero is presented as complete, with no notice | Probe at offset zero, as the label adapter does, through one shared helper |
| 4 | Protein cards cut features, isoforms and structures to five rows with no count or next step | Rows are dropped with a clean status | Add the total and a page step |
| 5 | OpenTargets drug paging counts accepted rows, not source rows | Paging repeats a row or stops early | Count source rows |
| 6 | Label search by application number removes rows and keeps the total | The rows and total disagree under a clean status | Record a notice or correct the total |
| 7 | The HTTP server exits on one failed accept or thread start, with nothing logged | A passing resource shortage ends the shared service and drops running calls | Log, pause briefly and keep serving |
| 8 | Nine idle connections make `/health` return 503 | The docs say a reader is reserved for health checks; it is not. A health-based restart could follow from idle sockets | Answer `/health` outside the reader limit, or give it a real reserved reader |
| 9 | A stored pool plus a newly disabled source gives Failed, not a rerun | A working search fails with exit 4 and no next step for up to a day | Treat that pool as a miss and search again |

Findings 11, 12, 15, 17 and 18 also fall under the every-known-fix stance, but none blocks on its own.

### Findings in rank order

#### 1. A partial article search is stored and served to everyone for 24 hours

- **Label:** Confirmed defect. **Severity:** High. **When:** before release.
- **Stance:** the 2026-10-23 stance ("A query pool is a stored snapshot, so paging is stable and a warm page makes no request") covers stored pages. It does not say a failure should be stored as the answer for new searches. The weigher should rule on that.
- **Broken promise:** a passing source failure should affect one answer, not a day of answers. The reply offers a `fresh` step, but hosted strips it.
- **Evidence:** `crates/biomcp-query/src/article/mod.rs:75-78` reuses a stored pool unless `--fresh` is set. Line 130 publishes after `sources::search` even when a source failed. `article/sources.rs:71,86` continues if any source answered. `store/pool.rs:79,111,142,155` sets the 86,400-second life and the `pool_current` pointer. `crates/biomcp/src/mcp/hosted.rs:135-138` drops `--fresh`, and `hosted.rs:215,227,310` removes disallowed follow-ups. README line 63 says hosted refuses `--fresh`.
- **Who shares it:** `store/ownership.rs` makes a pool private only for no-fallback key sources (AlphaGenome and OncoKB in the code I searched). Article sources (PubMed, Europe PMC, LitSense, PubTator, Semantic Scholar, LitVar) are not among them, so the pool is public to all hosted callers.
- **Structural cause:** the pool has one meaning, "the answer to this request", with no separate state for "an answer that is incomplete". Copies: `article/mod.rs`, `article/edges.rs:58,79`, `article/recommend.rs:63,82`.
- **Reproduction (j32005, ran):** `search article AKT1 --author Carpten --source pubmed,europepmc` with `tests/replies/article/fold/failed-pubmed` gave `status partial`, 1 item, pubmed `failed`, pool `034a1aee`. The same search with an empty recordings folder, then with `--offline`, returned the same pool, the same 1 item and the same stored failure, with no request made. Not run through a live hosted server; the hosted part is read from code.
- **Smallest fix:** still store the pool so its own pages stay stable, but do not set `pool_current` when any source failed or came from stale data. A follow-up page names its pool explicitly, so paging keeps working. The next identical search retries. Cost: a few lines per copy and more upstream calls during an outage, which quotas already bound. Test: run the case above, then a second search with healthy recordings must return full results and a new pool.
- **Ticket:** no open ticket fits. This needs a new one.
- **Uncertain:** how often partial article answers happen in production. Hosting logs would show it, but finding 12 says little is logged.

#### 2. A good trial page is reported as failed when a side call fails

- **Label:** Confirmed defect. **Severity:** Medium. **When:** before release.
- **Broken promise:** `crates/biomcp-query/src/trial.rs:49-54` plainly intends a partial result in this case. Line 93 overwrites it.
- **Evidence:** `trial.rs:38-47` folds a failed `/version` call or `count_error` into the source error. Line 93 `status::search_for` sees no answered source and sets Failed. `status.rs` `body_visible()` and `crates/biomcp/src/command.rs:229` then hide the body. `crates/biomcp-adapters/src/ctgov/mod.rs:177-188` is the count path, and `ctgov/mod.rs:37` caches `/version` for one day, which narrows the window but does not close it.
- **Reproduction (j32005, ran):** `search trial --condition "breast cancer" --sex FEMALE` against `tests/replies/trial/sex` gave exit 0, `complete`, 10 items. With `/version` changed to three 500 replies: exit 4, `failed`, no `result`, while the same reply says `returned 10`, `total 16889`, and still offers a `page` step.
- **Structural cause:** two owners decide the same status. The per-source step reaches partial, and the search-wide step ignores it. A sweep of the adapters found no copy in what it covered. Composite cards and several side calls were not reached (see checks skipped).
- **Smallest fix:** count a source with retrieved items as answered in `search_status`, or skip `search_for` when items exist. Cost: a small change with one replay case. Test: the case above must give exit 0 or the documented partial exit, status `partial`, 10 items.
- **Ticket:** none open.
- **Uncertain:** nothing about the behavior. Severity depends on how often `/version` alone fails.

#### 3. Paging past the end of an openFDA search gives a looser search or a false zero

- **Label:** Confirmed defect by reading. **Severity:** Medium. **When:** before release.
- **Broken promise:** `crates/biomcp-adapters/src/openfda_label/search.rs:113` says "Select by native total so exhausted pages never broaden the query", and lines 128-129 explain that openFDA reports an exhausted offset as a 404 with no total. The label adapter guards against this. Its neighbors do not.
- **Evidence:** `crates/biomcp-adapters/src/drugsfda/mod.rs:60-63` runs the exact search at the caller's offset and falls to the word-match search if the total is 0. `drugsfda/mod.rs:137` turns the 404 into total 0. So an offset past the exact matches returns word-match rows and the word-match total, marked complete, with no notice. The same 404-to-zero mapping sits in `faers/search.rs:93`, `maude/search.rs:91`, `fda_drug_recall/mod.rs:171`, `fda_device_recall/mod.rs:168`, `fda_510k/mod.rs:82`, and the label application-number branch at `openfda_label/mod.rs:133`. There a count of 500 becomes 0 on a later page. I read the Drugs@FDA, label and FAERS code myself; the other lines come from the sweep.
- **Structural cause:** each openFDA adapter maps "not found" on its own, and only one knows that "not found" can mean "offset ran out".
- **Reach:** BioMCP's own page steps stop at the total, so this needs a caller-supplied offset or a total that shrinks between pages. Agents do set offsets, so it is reachable through tools.
- **Reproduction:** not run. It rests on the project's own comment for the 404, not a live call. Settling test: fake the exact search returning 404 at skip=N and the word search returning rows; the reply must not list word-match rows as complete. A search with total 500 at offset 600 must not report total 0 as complete.
- **Smallest fix:** one shared helper for all seven adapters: when the page is absent and the offset is above zero, probe at offset zero for the true total, as `candidates` does. Cost: one extra request on an exhausted page only.
- **Ticket:** none open.
- **Uncertain:** the live 404 behavior on each openFDA endpoint.

#### 4. Protein cards cut sections to five rows with no sign of more

- **Label:** Confirmed defect by reading. **Severity:** Medium. **When:** before release.
- **Evidence:** `crates/biomcp-query/src/protein/sections.rs:33-52` `first` takes `section_limit` rows of features, isoforms and structures into a body holding only `items`. `protein/next.rs` names none of these sections, so a requested section gets no follow-up step. Domains and complexes in the same card do carry a total and steps.
- **Broken promise:** rows dropped with a clean status. The reader cannot tell five rows from five of thirty.
- **Reproduction:** not run. Settling test: an entry with 30 features, requested with the features section, must show a total of 30 or a step to the next page.
- **Smallest fix:** add the total and a page step, as the domain section does. A paged features command already exists, according to the sweep. Cost: small, plus a schema change, which the no-backwards-compatibility stance allows.
- **Ticket:** none open. The typed follow-ups ticket (0680) touches the same steps.
- **Uncertain:** nothing about the behavior. Whether five rows is a sensible default is a question for the owner, and I make no clinical judgment on it.

#### 5. OpenTargets drug search pages by accepted rows, not source rows

- **Label:** Confirmed defect by reading. **Severity:** Medium. **When:** before release.
- **Evidence:** `crates/biomcp-query/src/drug/native.rs:126` sets `received` from rows that survived `crates/biomcp-adapters/src/opentargets/map.rs:54-75`, which has already dropped refused rows. `crates/biomcp-adapters/src/native_window.rs:40` shows the contract: `received` counts source slots, refused ones included. The next offset then starts early and repeats a row. A page where all rows are refused reads as `received == 0`, and paging stops while the total says more.
- **Reproduction:** not run. Settling test: a page of 10 with 1 refused must give next offset 10, not 9; an all-refused page must still offer a next step.
- **Smallest fix:** count source rows after the skip. Cost: one line and one test. The page is already marked partial, so harm is limited to repeats and early stops.
- **Ticket:** none open.
- **Uncertain:** how often OpenTargets returns a row this code refuses.

#### 6. Label search by application number removes rows and keeps the total

- **Label:** Confirmed defect by reading. **Severity:** Low to Medium. **When:** before release.
- **Evidence:** `crates/biomcp-adapters/src/openfda_label/search.rs:87-95` filters rows after the fetch, with no problem recorded and the total unchanged. A reply can say total 3, show 1 row, and be complete. The window's offset also no longer matches what the caller saw.
- **Reproduction:** not run. Settling test: return 3 rows where 2 carry another number; the total and the rows must agree, or a notice must explain.
- **Smallest fix:** record a notice, or correct the total for the page. Small.
- **Ticket:** none open.
- **Uncertain:** whether the source ever returns such rows, and whether lowercase input reaches this case-sensitive comparison.

#### 7. The HTTP server exits on one accept or thread failure, with no message

- **Label:** Confirmed defect. **Severity:** Medium. **When:** before release.
- **Broken promise:** a shared service should shed load, not stop. Running calls are dropped without cancel or join, which breaks the careful joined shutdown elsewhere in this code.
- **Evidence:** `crates/biomcp/src/mcp/http.rs:142` returns on any accept error other than WouldBlock or Interrupted. Line 132 `set_nonblocking(false)?` and the `spawn(...)?` near line 168 do the same.
- **Reproduction (j32005, ran):** `serve --hosted --http 127.0.0.1:0 --http_max_connections 400` with a file limit of 96. After about 128 connection attempts the server exited with code 5 and an empty stderr, and new connections were refused. This setup is deliberately harsh: the connection cap is above the file limit. Not tested at the hosted team's 32 calls and 48 connections, with a normal limit, or on Linux.
- **Why it still matters:** the Linux `accept` manual says some network errors, such as a connection aborted before accept, must be treated as retryable. One such error here ends the service. Not observed by me.
- **Smallest fix:** on these errors, log the kind, pause briefly, and continue. Refuse only that one connection on a per-connection failure. Cost: about ten lines plus a test with a forced failure.
- **Ticket:** none open. Would fit the hosted validation work.
- **Uncertain:** how hosting restarts the process. A restart hides this but still drops every running call.

#### 8. Nine idle connections make `/health` and all requests return 503

- **Label:** Confirmed defect. **Severity:** Medium. **When:** before release.
- **Broken promise:** the configuration docs say readers are reserved so health checks can be classified.
- **Evidence:** `crates/biomcp/src/mcp/http.rs:125` allows `max_connections + 1` readers. A reader is held while `wire::read` waits up to the 2-second budget, before the request can be seen as `/health`.
- **Reproduction (j32005, ran):** default hosted server. Idle `/health` gave 200. With 9 open silent sockets it gave 503. Over 6 seconds, with 9 more sockets at 1.5 seconds: 16 answered, 7 refused.
- **Structural cause:** the server must read a request to learn that it is a health check, so a health check cannot have priority in the shared pool.
- **Smallest fix:** a separate small reader pool used only when the main pool is full, or take this claim out of the docs and raise the reader count. Cost: small. The front end may already limit this. If so, the fix can be the docs change alone.
- **Ticket:** none open. Hosted validation should test it, as a health-driven restart under slow clients would be the visible symptom.
- **Uncertain:** whether the front end holds idle connections to BioMCP. Question for hosting.

#### 9. A stored pool and a disabled source give Failed instead of a new search

- **Label:** Confirmed defect for the command line. Design gap for hosted. **Severity:** Medium. **When:** before release.
- **Evidence:** `crates/biomcp-query/src/article/page.rs:67-88` returns no items if any pool source is now disabled. `crates/biomcp-types/src/query/article.rs:380` `identity()` leaves source exclusions out of the key. The check fires on healthy pools too, since any reason other than Disabled trips it.
- **Reproduction (j32005, ran):** using the pool from finding 1, `--disable-sources pubmed` gave exit 4, `failed`, no items, no next steps, although Europe PMC's stored row was allowed and the disabled source had returned nothing. `--disable-sources europepmc` gave the same. No search was attempted.
- **Reverse direction (not run):** a pool made with a source disabled carries no marker, so a caller with wider rights gets the narrower result. In hosted, pools are shared across declared uses (`crates/biomcp/src/command/hosted_use.rs:53-75`), so a higher-tier caller's pool fails for a restricted caller for up to 24 hours, and a restricted caller's pool narrows a higher-tier caller's answer. This cannot occur while our deployment stays restricted (2026-11-04 stance), but the default ceiling is academic. Whether any article source is non-commercial or licensed was not checked.
- **Smallest fix:** add the sorted disabled-source set to the pool key in `identity()` handling, so a different exclusion means a different pool. This also removes the failing branch for new searches. Keep the branch for explicit pool IDs. Cost: a small change, a bit more storage. No migration, under the no-backwards-compatibility stance.
- **Ticket:** none open.
- **Uncertain:** copies in `edges.rs` and `recommend.rs` share the pattern by reading, but only article search was run.

#### 10. One caller can use all hosted capacity

- **Label:** Design gap. **Severity:** Medium. **When:** after release, if the front end does enforce shares; otherwise before.
- **Stance:** covered. Ticket 0548 records "Defers: per-user caps, which the hosted service enforces in front of BioMCP."
- **Evidence:** `crates/biomcp/src/mcp/http/call.rs:61` refuses at capacity with no queue or per-caller share. The test at `call.rs:173` shows timed-out workers keep their slots until they exit. Keyless hosted calls share one quota bucket (`crates/biomcp-fetch/src/quota/admit.rs`), so one caller's burst delays others at each provider, which the front end cannot see.
- **Reproduction:** not run. Settling test: two callers through the real front end, one bursting, measuring the other's refusals.
- **Smallest fix:** none here if hosting confirms the cap. The shared provider quota is the part a front end cannot fix. Name it in the hosted notes.
- **Ticket:** fits 0641 (repeated load tests).
- **Uncertain:** the front end's real behavior. Question for hosting.

#### 11. The hosted cache has no disk ceiling

- **Label:** Design gap. **Severity:** Medium. **When:** before hosted runs long-term; can follow 1.0 if the hosted team runs a regular clean.
- **Evidence:** `crates/biomcp-fetch/src/cache.rs:26,291` keeps replies for 30 days past freshness, each up to 8 MiB (`lib.rs` `FetchConfig`). Hosted refuses `store clean` as a tool (README line 63). No VACUUM appears in the docs or the code I read, so freed space is not returned. The SQLite write-ahead file has no size limit set.
- **Reproduction:** not run. Settling test: a soak with varied queries, tracking file size against a stated ceiling. Under the measured-caps stance the ceiling should come from measurement.
- **Smallest fix:** a configurable maximum cache size with oldest-first pruning, or a documented scheduled `store clean`. Whether the command-line clean is safe beside a running server is the first thing to test.
- **Ticket:** none open.
- **Uncertain:** hosted disk size, and whether hosted restarts on a fresh disk, which would make this moot.

#### 12. The hosted server tells operators almost nothing

- **Label:** Design gap. **Severity:** Medium. **When:** before hosted validation ends, as it makes every other finding harder to see.
- **Evidence:** only four diagnostic sites exist (`crates/biomcp/src/exit.rs:17,24`, `mcp/http.rs:103,184`). No call ID, duration, per-source result or counter. In finding 7 the server exited with nothing at all on stderr. `/health` (`crates/biomcp-query/src/store/mod.rs:201-215`) checks only owner metadata, so a damaged reply table would likely show healthy while calls fail, since `crates/biomcp-fetch/src/cache_request.rs:222` fails the request on a cache read error instead of going live.
- **Reproduction:** the silent exit ran (j32005). The damaged-table case did not. Settling test: damage the `reply` table, check `/health` and one call.
- **Smallest fix:** one structured line per call on stderr with tool, duration, status and source failures, no arguments or caller data. The privacy side belongs to the security reviewer.
- **Ticket:** none open; hosted evidence work is the nearest fit.
- **Uncertain:** whether the front end logs enough to cover this.

#### 13. Hosted loses partial answers at the time limit

- **Label:** Likely defect. **Severity:** Low to Medium. **When:** after release is defensible: it fails safely, with a clear timeout.
- **Evidence:** the command's deadline equals the whole-call scope in `crates/biomcp/src/mcp/http.rs` `tool()`, and `http/call.rs:132-141` returns a timeout once that scope reaches zero, even if the reply has arrived. Sources that had already answered are lost.
- **Reproduction:** not run. Settling test: a replay with one source delayed past the budget must return the fast source as partial.
- **Smallest fix:** give the command a deadline a little shorter than the call scope. Cost: one setting, chosen from measurement.
- **Ticket:** none open.

#### 14. A large cache write can stall reads and `/health`

- **Label:** Risk. **Severity:** Low to Medium. **When:** after release unless hosted load shows it; 0641 can measure it.
- **Evidence:** `crates/biomcp-fetch/src/cache.rs:155` `write_sql` holds the shared connection while waiting for the writer, and `cache.rs:385` `wait_mutex` is not fair. No SQL interrupt exists, so the busy timeout does not bound a running query. `cache.rs:254` `try_sql` treats a busy cache as a miss, so a store operation sends traffic upstream.
- **Reproduction:** not run. `make hosted-http-load` exists, and I found no recorded result. It stays as a settling test, run during a bundle publish.
- **Ticket:** 0641.

#### 15. A cache moved after an interrupted sync cannot sync that bundle again

- **Label:** Confirmed defect. **Severity:** Low. **When:** before release if cheap, under "1.0 ships every known fix".
- **Evidence:** `crates/biomcp-fetch/src/bundle/store.rs:50-66` `validate` returns an error if the stored part path has another parent or none. `claim` calls it on any old row, before any network use.
- **Reproduction (j32005, ran):** inserted a `bundle_sync` row for `hgnc` with part path `/nonexistent/old/parts/.tmpABCDEF`. `sync hgnc` gave exit 5, `local_error: filesystem`, with no attempt to connect. `crates/biomcp-fetch/src/cleanup.rs:120` lists `bundle_sync` as a transport table; I did not test whether `store clean` clears it. I ran one attempt, so "forever" rests on the row not being removed in the code.
- **Smallest fix:** treat a part outside the current folder as a dead claim: delete the row and start over. A few lines and one test.
- **Ticket:** none open.

#### 16. Performance on the hot path

- **Label:** Risk. **Severity:** Low. **When:** after release; these cost time, not correctness, and no budget is broken yet.
- **Evidence:** `crates/biomcp/src/command/transport.rs` `with_cancellation` builds new agents per hosted call, so there is no connection reuse. A new SQLite connection opens per store operation (`crates/biomcp-query/src/store/mod.rs`). Full-bundle prefix reads run per search (`crates/biomcp-adapters/src/ema/mod.rs:57`, `who_ivd/search.rs:17`, `who_pq/mod.rs:50`). `Cargo.toml` has no `[profile.release]`.
- **Reproduction:** not run. Settling test: time per hosted call, split by TLS setup, cache and provider time, in the 0641 load run.

#### 17. Docs disagree on a missing `x-biomcp-use` header

- **Label:** Inconsistency. **Severity:** Low. **When:** before release. Belongs to the docs reviewer.
- **Evidence:** README says a missing header means commercial. `docs/configuration.md` says restricted, and `crates/biomcp/src/command/hosted_use.rs:27` agrees with restricted.

#### Brief note for the gate reviewer

The nightly live check, job j31978 under ticket 0068, failed with exit 2 on this exact commit. Its evidence is on yellow at `/home/ian/workspace/data/biomcp/0068-scheduled/20261106T090113Z-cmtr4ay0`. I did not read why.

#### 18. Smaller correctness items from the adapter sweep

**Label:** as marked. **Severity:** Low. **When:** before release if cheap, under "1.0 ships every known fix"; none causes harm that justifies holding the release alone. All were found by reading, by a helper agent on my behalf, and nobody ran them. I checked only the CPIC item myself, so treat the rest as Likely until a failing test exists. No open ticket fits.

- Two next steps name the wrong source, so disabling an unrelated source removes them: `crates/biomcp-query/src/variant/recurrence.rs` (end of `execute`, labelled CancerHotspots, data from cBioPortal) and `crates/biomcp-query/src/pathway/relation.rs:96` (always Reactome). The filter is in `next.rs` `finish`.
- CPIC guidelines page by offset with no sort order, unlike every other CPIC read: `crates/biomcp-adapters/src/cpic/read.rs:220`. Missing `order` confirmed by me. Whether the server's order drifts is unknown.
- Protein lookup by gene skips a malformed UniProt ID with no problem recorded, and `rows.refusals` is never read: `crates/biomcp-query/src/protein/get.rs` `ids`.
- The EMA regulatory section shows a total above its rows but offers no next page: `crates/biomcp-query/src/drug/ema.rs:125-146`.
- GTR search can mark a clean page partial because of a non-matching row: `crates/biomcp-adapters/src/gtr/search.rs:55-57`.
- A missing author list reads as zero authors: `crates/biomcp-adapters/src/europepmc/mod.rs:182`, `semantic_scholar/search.rs:241`.
- The displayed trial offset drifts when rows are refused: `crates/biomcp-query/src/trial.rs` `apply_page`. Paging uses a token, so no rows are lost.

Checked clean by the sweep: stored-query keys for study, protein, GWAS, pathway, ORCID, author, VAERS, phenotype, FAERS and MAUDE get, trial get, NCI, article and pgx. Paging for GWAS, ClinVar, MyVariant, CIViC, SEER, OLS, Monarch and the article union. Trial and variant filters are all sent.

### Questions for hosting

- Does the front end enforce per-caller shares and hold idle connections away from BioMCP?
- What restarts the process, and does a failed `/health` trigger it?
- How much disk does the hosted cache have, and does it survive deploys?

No clinical questions came up in my area.

### Strengths

- Crates are layered cleanly, and status is typed rather than guessed from text.
- Partial answers were labelled in every run I made.
- A bundle publish is one transaction, with leases and a fenced holder.
- WAL startup retry is proven, and threads are scoped with no detached workers.
- Tool workers catch panics. The result cap refuses rather than cuts text short.
- HTTP reads, writes and body sizes are all bounded. Retries have ceilings and honor Retry-After.
- Hosted key-bound pools are private to the caller. There is no `unsafe` code.
- Stale and offline answers are marked.

### Checks skipped

- No live provider calls, no live hosted server, no front end.
- No Linux or Windows runs of the probes: only macOS on m5.
- No load run. `make hosted-http-load` and repeated-load tests were not run, so an idle machine's pass here proves nothing about concurrency.
- No disk soak, no power-cut test on WAL, no damaged-database test.
- Deslop and advisory scanners, gate history, dependency table and ticket judgments were left to the other reviewers.
- Adapter sweep did not reach: InterPro and Complex Portal mapping; study get, rows and files; cellosaurus and ChEMBL cell lines; biomarker and organization; ORCID works; variant assertions and identity; MyVariant measurements; gene sections for ClinGen, HPA and GenCC; the MyGene date mapping, mydisease and erepo; article get, evidence and semantic edges; trial get pages and NCI sites; MAUDE and FAERS get projection; ChEMBL search, drug native sections, MyChem and `label_bound`; allele registry coordinates. Some others were read only in part.
- `edges.rs` and `recommend.rs` copies in findings 1 and 9 were read, not run.

# Report B: security, data rights, dependencies

## BioMCP 1.0 adversarial review: security, data rights, dependencies (reviewer B)

### Snapshot

- Commit: main f33f42a55e040fccfc3394b516799f4baaf89133, read from a clean `git archive` export. Reviewed 2026-11-06.
- Tools: rustc 1.95.0, cargo-deny 0.19.4, cargo-audit 0.22.1, cargo-machete, gitleaks 8.30.1 (the repo's pin), RustSec database of 2026-11-05 (commit 26fd2d43, 1409 advisories).
- Jobs under 0688, on Yellow: j31997 (pm shows it passed, but my script tolerated a failed build: the sccache wrapper is missing on Yellow, a problem of my job, not the product, so its result counts for nothing) and j32003 (release build in 10m 13s, offline probes, scans; exit 0).
- No provider was called. KEGG and LitSense were not called. All probes used a loopback server with empty config and no keys.
- Read outside the snapshot, for context only: `biomcp-hosting` `docs/security.md`, `docs/sources-and-licensing.md`, and the `hosted_max_use`, `mcp_tool_mode` and source-disabling keys of `config/service.json`.

### Verdict

**Ready after the listed fixes.** I found no Critical finding, no secret leak, and no way across the MCP trust boundary. The code-level security work is strong. What blocks is data rights: one code gap (LitSense) and two questions only Ian can answer. `sdlc/decisions/2026-10-27-1-0-ships-every-known-fix.md` and `2026-11-05-everything-in-1-0.md` put every accepted fix in 1.0, so the "when" below defaults to before release unless a reason is given.

### Before release

| # | Finding | Why it blocks | Smallest fix |
|---|---|---|---|
| B1 | LitSense returns sentences from noncommercially licensed papers to hosted `restricted` callers | Breaks Ian's 2026-11-04 ruling that such data is not returned; the fixtures' own notes show these sentences exist | Drop `sentence` text when `disabled_use(Use::PerRecord)` is set, or disable LitSense in hosted `restricted` |
| B2 | Six sources are served in `restricted` while their own terms text says rights are unverified or need review | The ruling admits only sources whose terms allow commercial use. **Ian decides** | Ian rules per source; terms text then records the ruling |
| B3 | A complete WHO PQ export and live Semantic Scholar replies, both labeled `noncommercial`, sit in the public repo | Licensed data in what ships; the append puts it on the main public repo. **Ian decides** | Replace with synthetic rows, if Ian so rules, before the append (0294) |
| B4 | README's hosted section describes the retired two-mode policy | An operator gets false security guidance: wrong default and wrong effect of keys | Rewrite README lines 65 to 67 from `release/notes.md:38` |
| B5 | A dozen idle sockets refuse `/health` and every call; docs say health stays reachable | Reproduced; the health claim is false | Give framing readers real headroom, and correct the doc |
| B6 | The secret history scan refuses to run at this snapshot | No secret-history result exists for the commit; fails closed | Land open ticket 0687 |

### Findings in rank order

#### B1. LitSense sentences bypass the per-record rights check

- **Label:** Confirmed defect (plain code path). **Severity:** High. **When:** before release. The hosted service may already do this, so tell hosting now.
- **Promise broken:** `sdlc/decisions/2026-11-04-hosted-source-modes.md:5`: `restricted` serves "the commercial-permitted records of `per-record` sources", and Ian: "If there are data elements that are not usable for commercial entities, we will not access them, share them, or return them back."
- **Evidence:**
  - `crates/biomcp-adapters/src/litsense/mod.rs:18-22` labels the source `Open`, with terms "NCBI policies; publisher rights in input text"; line 16 says public access grants no rights in the sentences.
  - `litsense/mod.rs:69` returns each paper's sentence text with no license check.
  - The check exists for the other full-text routes only: `pmc_oa/rights.rs:15`, called from `pmc_oa/mod.rs:105`, `europepmc/core.rs:174` and `article/sections/pdf.rs:99`. `article/evidence.rs:119` restricts text the same way.
  - `tests/replies/article/litsense/README.md`: "CC BY-NC variants are excluded", and the unmodified replies stay outside Git. One capture kept 6 papers of 65.
  - `tests/cases/article/litsense/licensed-subset.yaml:3`: "live endpoint cannot enforce this subset without retrieving excluded sentences."
  - `biomcp-hosting/config/service.json` sets `hosted_max_use: restricted` and does not disable LitSense. Its `docs/sources-and-licensing.md` says mixed-data filtering belongs to core.
- **Structural cause:** Fixing one copy. The per-record rule was built at the full-text readers, not at every route that returns article text. The team kept these sentences out of Git but not out of hosted replies.
- **Reproduction:** not run, because LitSense must not be called. Settling test: a recorded-reply hosted case at `restricted` whose LitSense reply holds a sentence, asserting no `sentence` text reaches the caller. It needs no live call.
- **Fix:** in `litsense::search`, keep the PMID ranking and omit `sentence` when `disabled_use(Use::PerRecord)` is set, and relabel the source `per-record`. About 10 lines plus a case. It gives up sentence previews in hosted mode. Checking each PMID's license instead keeps the previews for open papers, at one extra request per paper. Check stored pools too: `article/page.rs:66-90` filters them by disabled source, not by use, so sentences already stored may need their own test. No open ticket fits; 0676 is the nearest hosted ticket but covers schema only.
- **Uncertain:** how much of the live corpus is NC. I did not check whether hosted revision 4 contains this path. PubTator carries the same terms text, and I did not trace whether it returns passages.

#### B2. Terms text and use labels disagree, and the label decides what hosting serves

- **Label:** Question, plus Inconsistency. **Severity:** High until Ian rules. **When:** before release. **Stance:** ticket 0585 records that labels follow the hosting team's m22678 summary. Ian's own words in that ticket (m22685) cover only FAERS, MAUDE and VAERS.
- **Evidence:** each source below is admitted in `restricted`, because `hosted_use.rs:44-48` and `:53-75` block only `Noncommercial`, `Licensed` and non-commercial per-record items.

| Source | Label | Terms text in the registry | Where |
|---|---|---|---|
| cBioPortal | attribution | "study-specific terms unverified; commercial restrictions may apply" | `cbioportal/mod.rs:23-24` |
| Cancer Hotspots | open | "data reuse unverified" | `hotspots/mod.rs:16-18` |
| ClinGen eRepo | open | "record rights unverified" | `erepo/mod.rs:14-16` |
| ClinGen Allele Registry | open | "output rights require review" | `allele_registry/mod.rs:19-21` |
| Mutalyzer | open | "service output terms require review" | `mutalyzer/mod.rs:16-18` |
| VariantValidator | open | "service output terms require review" | `variantvalidator/mod.rs:18-20` |

  Also, `response/recurrence.rs:56-57` and `variant/recurrence.rs:32` tell the caller rights are `unverified: true`, and `docs/cautions.md` repeats it for Hotspots and cBioPortal. Hosting bakes the cBioPortal study list into its image. PubMed abstracts, LitSense and PubTator are `open` with "publisher rights" retained; Monarch, MyDisease, OLS and GWAS are labeled on source-specific terms.
- **Structural cause:** Truth restated. The label and its terms text are two facts, and the evidence that reconciled them (m22678's verbatim terms) is not in this repo. `Use` has no "unreviewed" value, so an unresolved source must be labeled as allowed or blocked. It became allowed.
- **Reproduction:** read from code; no run needed.
- **Fix:** Ian rules per source. Then rewrite the terms text to the ruling, and add one test that fails when a source served in `restricted` carries "unverified", "require review" or "may apply". Removing a source costs hosted coverage. No open ticket.
- **Uncertain:** whether the terms text is stale or the label is wrong. I did not judge any source's actual terms.

#### B3. Real data from `noncommercial` sources is in the public repository

- **Label:** Question. **Severity:** High until Ian rules. **When:** before the append (0294). **Stance:** `AGENTS.md` says "Respect source redistribution terms". No decision covers captures from noncommercial sources.
- **Evidence:**
  - `tests/cases/drug/who-dates.yaml`: "Complete unedited finished-product CSV captured" on 2026-11-04, at `tests/replies/drug/who-dates/who-pq/` (about 130 KB). WHO PQ is `Noncommercial`, "export-specific reuse grant unverified" (`who_pq/mod.rs:14-16`).
  - 21 non-synthetic Semantic Scholar files, about 215 KB, such as `tests/replies/article/edges/live/semantic-scholar/`. The source is `Noncommercial`, under its API License Agreement (`semantic_scholar/mod.rs:12`).
  - Lesser cases: 7 cBioPortal files (440 KB, see B2) and 1 VAERS file (338 KB, which m22685 appears to cover).
  - Contrast: OncoKB, DDInter, WHO IVD and HPA fixtures are synthetic. The CPIC fixture README says real data stays outside Git. `biomcp` is PUBLIC (checked with `gh`), and the root LICENSE is MIT with no carve-out for `tests/replies`.
- **Structural cause:** Fixing one copy. 0476's audit named specific datasets and its check (`scripts/article_licenses.py`) covers article text only. Nothing checks replies from sources the registry labels noncommercial, and the WHO capture landed after that audit.
- **Reproduction:** counted from the export. My split of live and synthetic relies on directory names and READMEs, so counts are approximate. ORCID fixtures are synthetic controls; I excluded them.
- **Fix:** if Ian rules against shipping, replace the WHO capture with authored rows and the Semantic Scholar captures with synthetic ones, and extend the license check to fail on a non-synthetic capture of a `noncommercial` or `licensed` source. The data stays in existing public history unless he also orders otherwise. Fits 0294's preconditions.
- **Uncertain:** whether the wheel, image or other artifacts hold `tests/` (not packed here; by design they should not).

#### B4. README's hosted section is the retired policy

- **Label:** Inconsistency. **Severity:** Medium. **When:** before release.
- **Evidence:** `README.md:65` says "A missing header means commercial", lists only `commercial` and `academic`, says commercial disables "noncommercial or restricted" sources, and says "A caller source key enables that source for either use". The code does otherwise: `command/hosted_use.rs:27` maps a missing header to `Restricted`, and `hosted_use.rs:66-73` enables only sources with `Required` keys, and only in `commercial`. README omits `hosted_max_use`. `docs/configuration.md:86` and `release/notes.md:38` are correct.
- **Structural cause:** Truth restated: the policy is written in README, `docs/configuration.md` and the release notes, and only the last two were updated.
- **Reproduction:** j32003 showed the real behavior: a bogus value gets 400, and `commercial` and `academic` get 400 under a `restricted` ceiling.
- **Fix:** rewrite those lines, linking to `docs/configuration.md`. Minutes of work, no cost. Reviewer C's docs area may overlap.

#### B5. A few idle sockets take the server down, including `/health`

- **Label:** Confirmed defect (reproduced). **Severity:** Medium. **When:** before release for the false doc; the code fix follows the same ruling. Hosting's core sits on loopback behind its wrapper, so the exposure there is smaller.
- **Promise broken:** `docs/configuration.md`: framing readers "reserve a reader to classify health checks" and "`/health` uses neither the connection cap nor the tool call cap".
- **Evidence:** `mcp/http.rs:125` sizes readers at `max_connections + 1` (9 by default); `http.rs:154-161` answers 503 when none is free; `wire.rs:9` holds a reader up to 2 s; default capacity is 8 (`config.rs:15`). The reserved reader is available to any socket, since the path is unknown until the head is read.
- **Reproduction:** j32003, default config: 12 silent TCP connections, then `GET /health` returned 503 in 1 ms and a tool call returned 503 in 20 ms. It recovered once the 2 s read budget ran out. By arithmetic, not measured, an attacker holds this with about five new sockets per second and almost no bandwidth. An orchestrator reading 503 from health may also restart the task.
- **Structural cause:** Unshared state: one counter serves both unauthenticated reads and the health guarantee.
- **Fix:** make the reader pool much larger than the connection cap (threads waiting on a 2 s budget are cheap), keeping the worker cap. Small change, slightly more threads under attack. It raises the cost of the attack but does not end it, so also change the doc. No open ticket fits; the two that built this edge, hosted HTTP edge (0530) and hosted capacity settings (0548), are complete.

#### B6. The secret history scan does not run at this snapshot

- **Label:** Confirmed defect (reproduced). **Severity:** Medium. **When:** before release. Reviewer C's gate area; noted because it decides secrets evidence.
- **Evidence:** j32003: `release/secrets.py history . --since ...` stopped with `history-ignore-unreachable-commit` at row 178. Row 178 binds commit 450cc94, which is not an ancestor of f33f42a55. It fails closed, which is right. Open ticket "Ignore ledger binds to landed main" owns rows 178 to 181.
- **Cause and fix:** ledger rows bound to an unlanded commit. Ticket 0687 fits and is sound.
- **Related, not run by me:** pm's status for the final rehearsal (0293) records 12 unclassified `generic-api-key` findings in compiled AMD and Windows artifacts on an earlier candidate, values never inspected. Nobody has classified them, so that scan's result needs to be settled at the final candidate.
- **Substitute check:** gitleaks over the exported tree found 49 hits, all `generic-api-key`: 38 in `tests/replies`, 11 in `tests/fixtures`, none in code, docs, scripts or release. Earlier reviewed rows class this rule's hits as public identifiers, but I did not match these 49 to the ledger. The tree scan does not cover history.

#### B7. `--hosted` without a config file permits `academic`

- **Label:** Risk. **Severity:** Low. **When:** never, unless the weigher disagrees. **Stance:** 0619 sets the `academic` default on purpose so self-hosters keep behavior; our hosting pins `restricted` and `infra/lib/config.ts:238` refuses anything else.
- **Evidence:** `DeclaredUse` defaults to `Academic` (`hosted_use.rs:11-12`). j32003 accepted `x-biomcp-use: academic` on default config. A lost config mount widens the ceiling silently; only a caller's header can use it, and the wrapper strips that.

#### B8. `hosted_max_use` is ignored on hosted stdio

- **Label:** Confirmed defect (plain code path). **Severity:** Low. **When:** after release is defensible, since stdio has no remote caller; the fix is a few lines.
- **Evidence:** stdio builds `Context::new` with `declared_use: None` (`mcp.rs:203`, `context.rs:31`), and `hosted_use::restrict` returns the fetch unrestricted for `None` (`hosted_use.rs:35-43`). So `serve --hosted` over stdio with a `restricted` ceiling serves noncommercial sources. README:63 says hosted mode applies on both transports; the docs scope the ceiling to HTTP.
- **Reproduction:** not run. Settling test: stdio `serve --hosted`, ceiling `restricted`, a DDInter call on recorded replies must report `disabled`.
- **Fix:** default stdio hosted calls to the configured ceiling, or state the scope in the README.

#### B9. Any header value earns its own pacing bucket

- **Label:** Risk (plain code path). **Severity:** Low. **When:** after release: 0505 chose per-key buckets, and the wrapper only forwards keys the user stored.
- **Evidence:** `quota/admit.rs:8-14` and `quota/limiter.rs:50-53` key buckets by fingerprint with no check that the key is real. A `--hosted` server without the required front end lets a caller send a new value per request and escape shared pacing, which risks a provider blocking the server's address for everyone. `limiter.rs:42-46` also scans all buckets on each acquire. README:67 does tell operators to strip these headers.
- **Reproduction:** not run. Settling test: a stub provider with N calls bearing N random keys counts requests against the keyless rate.
- **Fix:** add a global per-source ceiling across hosted buckets, if the weigher wants one.

#### B10. A tag push is the only control over publication

- **Label:** Risk. **Severity:** Low. **When:** before the tag, as an account setting. **Stance:** `2026-11-01-no-manual-release-pause.md` rules out approval steps, so Ian's approval is a rule for people, not a mechanism.
- **Evidence:** `release.yml:6-7` runs on `v*` tags, and `docs/development.md:65` admits release tags with no reviewers. The admin settings list holds no tag ruleset limiting who can create `v*`. I did not read the actual settings.
- **Fix:** a tag ruleset on the public repo. This changes a third-party setting, so it is Ian's call.

#### B11. Small items

- **Likely defect, Low:** AlphaGenome's Apache-2.0 protos compile into the binary, but `release/notices.py` draws on cargo-about, which sees crates only, and neither it nor `license_check.py` names them. Not run. Settling check: search a staged `THIRD-PARTY-NOTICES.txt` for AlphaGenome.
- **Inconsistency, Low:** `BIOMCP_SQLITE_DIAGNOSTICS` (`sqlite_diagnostic.rs:5`, read at `observe.rs:339`) is read by the shipped binary and appears in no doc. It appends to a path the operator sets; I did not check what it writes. Document it or compile it out.
- **Design gap, Low:** no public doc warns that query text goes to third-party sources and stays in the cache URL column. BioMCP serves cancer care, so a line saying not to enter patient identifiers is cheap.
- **Question, Low:** NCI's comment and terms say it needs the user's own API key (`nci/mod.rs:23,37`), while its key tier is `Fallback` (`:30`). Ticket 0503 allows that, but `sdlc/decisions/2026-10-24-keyed-sources-run-only-with-the-user-s-key.md` reads differently if NCI's terms bind the key to its holder.
- **Inconsistency, Low:** `docs/cautions.md` says CPIC follows the registry's noncommercial ruling and the release notes say noncommercial use loses it, but `cpic/mod.rs:14` labels it `Open`.
- **Risk, Low:** a local build holds 368 absolute `/home/ian/.cargo` paths, and the release scripts set no path remapping, so a release binary would hold CI runner paths. No private names were found. Not run on a CI build.
- **Risk, Low:** apt packages in `release/Dockerfile` are pinned by version with no dated archive, so an old pin can vanish and break a rebuild.
- **Question, Low:** the BioData pin 8bcd2456 matches the `v1.0.0` tag but differs from 08d9399 named in the decision, which asked for a recheck. I found no record of that recheck in 0459.
- **For reviewer A:** non-hosted `serve --http` skips `Calls::run` (`http.rs:347-362`), so `http_max_calls` and the whole-call deadline do not apply, although the docs state the cap without that limit.

### Questions for Ian

1. Must LitSense sentences from non-CC-BY papers stay out of hosted replies (B1)? I read the 2026-11-04 ruling as yes.
2. May cBioPortal, Cancer Hotspots, eRepo, Allele Registry, Mutalyzer and VariantValidator stay in `restricted`, given their own unverified terms text (B2)? Are PubMed abstracts covered too?
3. May the complete WHO PQ export and the live Semantic Scholar replies stay in the public repo and go to the main public repo at the append (B3)?
4. Should a tag ruleset protect `v*` on the public repo (B10)?

### Dependencies

Scans:

- `cargo audit`: 0 vulnerabilities across 318 locked crates, fresh database. The tensor generator's separate lock (51 crates) is also 0.
- `cargo deny --locked check licenses advisories bans sources`: all ok, with 4 duplicate warnings (base64, getrandom, syn, windows-sys). A closed idea already declined the base64 change.
- `cargo machete`: no unused dependencies, on the root workspace.
- No advisory is ignored in `deny.toml`. No runtime exposure and no upgrade needed.

| Direct dependency | Job | Judgment under the dependency ruling |
|---|---|---|
| ureq, rustls, rustls-platform-verifier, rustls-webpki | Source HTTP and TLS | Keep; security critical, webpki pinned for a typed error |
| tonic, prost, h2, hyper, tokio, tokio-rustls | AlphaGenome gRPC only | Keep; heaviest branch, serves one source; a closed idea declined own framing |
| zstd | AlphaGenome tensor decompression | Keep, bounded |
| rusqlite (bundled) | Cache and store | Keep |
| serde, serde_json, serde_ignored, toml, serde-saphyr | Data and config | Keep; saphyr is build-time |
| schemars, jsonschema | Schemas | Keep |
| clap, shlex, winnow | CLI grammar and parsing | Keep; winnow reach not traced |
| httparse | HTTP framing | Keep, avoids own parser |
| quick-xml, csv, calamine, zip, flate2, lopdf | Source formats | Keep, each with limits checked in review |
| url, idna_adapter | Origin checks | Keep; a closed idea declined replacing url |
| sha2, base64, httpdate | Hashes and encoding | Keep; small |
| ctrlc, rpassword, tempfile | Signals, key prompt, atomic files | Keep; closed ideas declined replacing ctrlc and rpassword |
| biodata | Value types | Keep; git rev 8bcd2456 equals the peeled `v1.0.0` tag (checked with `ls-remote`) |
| rcgen, time, libtest-mimic | Tests only | Keep; not shipped |

Candidates to drop: none. The ruling is met.

### Tickets

Only those in my areas:

- **0687:** sound; needed for B6.
- **0676:** sound; it does not address B1, which concerns returned data, not schemas.
- **0294:** needs correction: add Ian's answer to B3 as a precondition.
- **0293, 0640:** left to reviewer C.

### Strengths

- **Tool-call authority:** j32003 tried `--observe`, `--config`, `--record` and abbreviated forms, `skill install`, `serve`, `key`, `sync`, `store clean`, `--fresh` and `--save`, plus injection through `fetch` and `search`. All were refused or ignored, and no file was created.
- **HTTP edge:** Origin, wrong Host, duplicate Host, wrong port, absolute-form targets, HTTP/1.0, Content-Length with chunked, and an oversized body were all refused.
- **Secrets:** keys stay out of cache identities and are added only at the wire (`wire_request.rs`). Replies reflecting a key are rejected (`containment.rs`). Redirects are off for keyed agents (`network.rs:174`). Key files and the cache database are 0600 (confirmed in j32003). Error text is generic. Help and schema replies leaked no paths.
- **Caller isolation:** no-fallback replies are scoped by key fingerprint in the cache and in pools, evidence and snapshots (`privacy.rs`, `store/ownership.rs`).
- **Outbound:** declared origins per source, encoded path segments, a public-address check on each redirect and at the resolver for publisher downloads.
- **Parsers:** limits before work on gzip, zip, XLSX and PDF expansion and on bundles.
- **Supply chain:** actions pinned by commit, images by digest, apt packages by version, tools by version and checksum; least-privilege workflow permissions; Trusted Publishing, attestations and OIDC; an artifact secret scan.
- **Licensed data:** fixtures for OncoKB, DDInter and CPIC are synthetic, and the earlier audit's private datasets stay out. A private-name search of the binary was clean; the hits were false matches such as "aggregator".

### Skipped checks

- Live source calls of any kind, so B1's live share is unmeasured. I did not record which MCP tool mode the probes ran in, so treat `typed` mode, which hosting uses, as not probed.
- Packing and inspecting the wheel, image and archives (reviewer C's install pass; I read their scripts only in part).
- Hosted stdio run (B8), key-pacing run (B9), and the full test suite.
- Gate history and gate time; deslop scanners.
- Windows and macOS behavior, including key file permissions.
- GitHub repository settings, the wrapper's code, and the deployed service.
- Matching the 49 gitleaks hits to the ledger; full-history scan.
- Fuzzing the HTTP parser or `lopdf`.
- Clinical correctness; judging any publisher's terms.

# Report C: release, gates, tests, docs, plan, tickets

## BioMCP pre-1.0 adversarial review: release, gate trust, tests, maintainability, docs, plan and tickets

Reviewer C of three. Written 2026-11-06. Correctness, reliability, performance, security, data rights and dependencies belong to the other two reviewers and appear here only as short referrals.

### Snapshot

- Repository: `biomcp`, main commit `f33f42a55e040fccfc3394b516799f4baaf89133` ("Record landed ignore-ledger repair"), reviewed from an export, not the shared checkout. The only later commit, b886c33, adds the ticket for this review.
- Version in Cargo: 1.0.0. 20 open tickets including this review's own.
- Tools seen: cargo-deny 0.20.2, cargo-machete 0.9.2, actionlint 1.7.12. Python is pinned to 3.12 in `toolchain.json`; m5's default `python3` is 3.9.6; my jobs used 3.13.17.
- Every build and test ran as a pm job under ticket key 0688 on m5 (j32005, j32009, j32010, j32013, j32021, j32033). Nothing ran on Gator. Nothing was pushed or dispatched. Hosted run facts come from read-only `gh` and pm records.

### Verdict: not ready

BioMCP is not ready to publish today, and the owner has already said so: the hosting-validation decision holds the main preflight and the release cut. The release machinery itself is unusually careful, and the gap to ready is short and known. Four things stand between main and a release: main's own secrets check is red, the installed proof is red on main, no full release run has been green since the tooling changed, and nobody owns the publishing-account setup. All four have small fixes. None needs a redesign.

### Fix before release

| # | What | Cost | Owner ticket |
|---|---|---|---|
| 1 | Land the ignore-ledger repair so main's secrets check passes | Hours; branch exists | Ignore ledger binds to landed main |
| 2 | Land the Markdown help fix and put the tool proofs in the routine gate | Hours, about 30 s of gate time plus a compile | Markdown help lists every flag |
| 3 | Own and fix the 12 unclassified findings in the staged-artifact scan, and add stage, staged scan, signing and staged Homebrew to the section list | Half a day to a day; needs a real look at what matched | Release sections pass on main before the cut |
| 4 | Name an owner and add a read-only preflight for the publishing settings (`release-publication` environment, PyPI trusted publisher, `HOMEBREW_TAP_TOKEN`) | Half a day, plus Ian's approval for account changes | Append this repository's history |
| 5 | Raise the 120-minute qualify limit, or run the full suite once per platform | Minutes for the limit, half a day for the suite change | Rehearse the final release candidate |
| 6 | Correct the README's commercial instruction and hosted paragraph | An hour once the right groups are decided | New small ticket, or the docs fix under the release rehearsal |
| 7 | Move load tests before the final rehearsal | Edit ticket dependencies | Real-time and multi-process tests |
| 8 | Fix the plugin's unexpanded `{{front-map}}` | Hours | New small ticket |
| 9 | Remove internal wording from public release notes and the public candidate file | An hour | Rehearse the final release candidate |
| 10 | Bind the blind comparison and hosting hold to the publish ticket's dependencies | Edit tickets | Append this repository's history |

### Findings in rank order

Ranked by severity, then by area order (release, gate trust, tests, maintainability, docs, plan). "Reproduced" means I saw it in a job or a read-only record during this review. "Not run" means nobody ran it.

#### 1. No full release run has been green since the last rehearsal, and the current blocker has no owner

- Label and severity: Confirmed defect, High. Fix before release.
- Evidence: the last end-to-end pass was run 41938765815 (2026-11-05, commit ed9c36a). Ordinary run 42051185154 passed the candidate and all five native platforms, then failed at "Scan every staged publication artifact" with 12 unclassified `generic-api-key` findings across the AMD and Windows artifacts. This comes from the release ticket's progress record; I did not read the matched values or the retained Yellow evidence. `release/secrets.py:427` tolerates only the rule `private-key`, as recognised by `release/public_fixtures.py`, so a `generic-api-key` match has no classification path. Section runs 42095109380 and 42139916605 also failed.
- Structural cause: the scan's only exception path was built for one known rule, and compiled binaries contain whatever text the crates contain, so every new dependency or string can trip it. Copies: the section list in the "Release sections pass on main before the cut" ticket omits stage, staged scan, attestation and staged Homebrew, so the "each section passes alone on main first" ruling (`decisions/2026-11-05-release-finds-no-bugs.md`) does not cover this stage.
- Reproduction: hosted record read, not rerun (dispatch forbidden). Settling test: run the scan section on a candidate from main and classify each finding by rule, file and offset, without printing values.
- Smallest fix: add the stage and scan sections to that ticket, and classify the 12 findings by exact binary offset or a narrowly reviewed rule.
- Uncertainty: the 12 may be harmless metadata, or may not. No one has classified them. Reviewer B should treat them as unknown.

#### 2. Publishing settings have no owner and no check before the first public write

- Label and severity: Design gap, High. Fix before release.
- Evidence: read-only `gh api` shows `genomoncology/biomcp` environments are `biomcp-release-signing`, `github-pages`, `production`, `pypi`, `release`. There is no `release-publication`, which `docs/development.md` and `release.yml` require, along with a PyPI trusted publisher for that environment and `HOMEBREW_TAP_TOKEN`. The ticket that set this up is complete and the history ticket says "Changes: none".
- Why it matters: in `release.yml`, PyPI waits on GHCR, and Homebrew and the installer wait on GitHub. A missing setting would fail after the GitHub release and container are already public, leaving a half-published 1.0. PyPI versions cannot be reused. Rollback text covers promotion only.
- Structural cause: outside-account state is checked only by the job that needs it, at the moment it needs it.
- Reproduction: environment list read; failure path not run. Settling test: a read-only preflight job listing environment, trusted publisher and secret names, without values.
- Smallest fix: name an owner in the history ticket and add the preflight as the first step of the tag run. Changing the accounts needs Ian's approval under the third-party rule.
- Uncertainty: the tag run may be carried by a different environment setup after the history append. I saw only the current state. Whether the secret exists was not read in full.

#### 3. Intel Mac qualification finished 16 seconds inside its 120-minute limit

- Label and severity: Risk (measured), High. Fix before release.
- Evidence: run 42051185154, 15:07:59 to 17:07:43, 119m44s. Build 32.1 min, installed gate 86.2 min. Limit at `.github/workflows/release.yml:96`, and a copy at `.github/workflows/recover-section.yml:34`. An earlier run took 101 min. The build cache is off for tags (`release.yml:120`), so the real tag run gets no help from it.
- Structural cause: `release/gate.py` `installed()` runs the full case suite of about 611 trials for each of six channels, although it first proves each channel's binary has the same hash as the candidate. The rehearsal ticket asks only for version, health, one search and get, and stdio and HTTP MCP per channel.
- Reproduction: hosted record read. Settling test: the tag-mode run time on Intel.
- Smallest fix: raise both limits to 180 minutes, which costs nothing on a public repository. Better: full suite once per platform, short check on the other channels, saving about an hour.
- Uncertainty: hosted Intel machine speed varies; that is what makes the margin unsafe.

#### 4. The README's commercial instruction switches off PubMed, ClinicalTrials.gov and ClinVar

- Label and severity: Confirmed defect, High. Fix before release.
- Evidence: `README.md:20` tells commercial users to set `{"disable_sources": ["noncommercial", "restricted"]}`. Installed wheel with that config (j32009): 25 of 60 sources disabled, including `pubmed`, `ctgov`, `clinvar`, `europepmc`, `mygene`, `gwas`, `oncokb`. `biomcp search article AKT1 E17K` exits 4 with status failed. `biomcp search trial --condition melanoma` exits 4 with status failed. `biomcp get gene KRAS` is only partial. The README's own opening examples fail under its own instruction.
- Structural cause: `restricted` is a retired source group kept as an alias in `crates/biomcp-adapters/src/registry/legacy.rs`, and it means something different from the hosted `restricted` use level. Other copies: `crates/biomcp-fetch/src/lib.rs:121,197,283,315-322,474`, `crates/biomcp/src/config.rs:224-226,432`, `registry.rs:5,57`, `component.rs:20-39`, `myvariant/components.rs:99-123`, `release/notes.md:40`. This also keeps a compatibility alias against the owner's "no backwards compatibility in CLI or MCP" stance (2026-11-01).
- Reproduction: reproduced in j32009. No check catches it: changing README prose left `check_docs` at exit 0.
- Smallest fix: state the correct groups for commercial use, delete the alias, and add one README case that runs the documented commercial config. Which sources a commercial user must disable is a licensing question for Reviewer B and Ian; I judge only that the instruction and the examples contradict each other.
- Uncertainty: whether disabling these sources is the intended legal outcome. If it is, the README must say so plainly.

#### 5. Main's own secrets check is red

- Label and severity: Confirmed defect, High. Fix before release.
- Evidence: `make secrets` exits 2 with `{"code":"history-ignore-unreachable-commit","row":178}` (j31992, also j30917, j31795). Raised at `release/secrets.py:296-297`. Rows 178 to 181 of `release/secrets.ignore.jsonl` name commit `450cc94fb`, which is not an ancestor of main. The snapshot commit's subject says the repair landed, but it adds only the ticket; the fix on branch b90115661 is unmerged. That ticket is not among the release rehearsal's dependencies.
- Structural cause: rows bind to a ticket-branch commit that squash landing throws away. Whether the check passes depends on which refs a job's clone happens to hold: j31754 passed on the same content. Any new row will repeat this.
- Reproduction: reproduced in job records. Settling test: run `make secrets` in a fresh clone of main with only main's refs.
- Smallest fix: land the repair and add it to the rehearsal's dependencies. Then bind rows to file content or rewrite them at landing. The ticket's own gate j31824 failed on the flake in finding 7.

#### 6. The installed proof is red on main, because the routine gate never runs it

- Label and severity: Confirmed defect, High. Fix before release.
- Evidence: `crates/biomcp/Cargo.toml:49-51` sets `[[test]] name="tools" test=false`, so `--proofs` runs in neither `cargo test` nor the landing gate. It runs only in `release/smoke.py:150-152`. At the snapshot, `schema-discovery/rejects-bad-cases` fails with `expected text "--format"` (j32021, j32033); `help author --format md` is 775 bytes with no `--format` and no `--disable-sources` (j32010). Building and installing a real wheel and running `release/gate.py` exits 1 on m5 (j32009). So main cannot pass the release's own installed gate on any platform.
- Why it matters: this breaks the "release finds no bugs" ruling. The Markdown help bug was found by a hosted release section, not by the routine gate.
- Structural cause: release-only proofs. Copy: the Markdown help ticket's "passes in the routine gate" cannot be proved, because the routine gate does not run it.
- Reproduction: reproduced. Two other proofs failed in my first run only because I did not set `BIOMCP_TEST_PYTHON`; those are my invocation, not product defects. Adding the proofs to the gate must set it, as `scripts/trials.py:79` does.
- Smallest fix: add the proofs to the landing gate. About 30 to 60 seconds of run time plus a compile.
- Uncertainty: I attribute the installed gate's exit 1 to this proof because the tool proofs run first and this one fails, but my kept log held only the tail of that run. The tail showed fixture-teardown messages (`phase=TLS flush`), which may be normal proof output or a second cause. Settling test: rerun the installed gate after the help fix.

#### 7. Race and load tests are ordered after the final rehearsal

- Label and severity: Inconsistency, Medium. Fix before the cut.
- Evidence: "Real-time and multi-process tests run under repeated load" depends on the rehearsal, and "Moderate-risk timing tests" depends on it in turn, so races are hunted after the candidate is final. That contradicts the 2026-11-05 ruling. Live evidence: j31824 failed `gene-sections/gencc-first-publisher-deadline` at c84735fb0 under load 9.6 on 18 cores, with reply `"local_error":"busy"`, gencc `state: failed`, exit 5. The same test appears in the timing record. It passes elsewhere. Referred to Reviewer A for the `busy` result.
- Structural cause: order set when the rehearsal was the last thing, before the ruling.
- Reproduction: job record read; not rerun. Settling test: run the case 50 times under load.
- Smallest fix: reverse the dependencies. No code.

#### 8. Publication code first runs at v1.0.0

- Label and severity: Risk, Medium. Reduce before release.
- Evidence: rehearsals keep no-op publish steps, as the owner's stance asks. So `gh release create`, `docker manifest create/push`, PyPI upload of the hand-built wheel, `mcp-publisher`, and the gh-pages commit in `release/publication.py` have never run for real. Tests cover refusal paths and promotion. `release/artifacts.py` builds the wheel by hand (Metadata 2.1) with no `twine check`. `release/published.py` checks only that `biomcp X.Y.Z` answers: it has no plugin or desktop extension check, and the installer check does not compare with the staged bytes.
- Reproduction: not run. Settling test: `twine check` on a wheel; a push of the manifest step to a scratch registry; a TestPyPI upload if Ian approves.
- Smallest fix: add `twine check` to the stage section (minutes) and a byte comparison for the installer. Name the remaining unproved commands in the release ticket, as the no-op stance requires.

#### 9. Plugin users get a raw `{{front-map}}` marker

- Label and severity: Likely defect (traced in code, not run), Medium. Fix before release.
- Evidence: `skills/SKILL.md` holds a literal `{{front-map}}`, expanded only in `crates/biomcp/build.rs:328-334` for the binary. `plugin()` in `release/workflow.py` copies `skills/` raw, and the marketplace serves the repository root. `test_workflow.py:81` uses a fake guide, so no test sees it. Plugin proof checks only MCP `help`.
- Reproduction: traced in code; not run in Claude. It is also unverified whether `.claude-plugin/plugin.json` `"skills": ["./skills/"]` finds a `SKILL.md` directly in that folder. Settling test: install the plugin in a clean client and read the skill.
- Smallest fix: write the expanded skill in the plugin build and commit a generated copy for the root marketplace, or remove the marker from the shipped file.

#### 10. The README hosted paragraph contradicts the code

- Label and severity: Confirmed defect, Medium. Fix before release.
- Evidence: `README.md:65` says a missing header means commercial. `crates/biomcp/src/command/hosted_use.rs:25-30` maps a missing header to restricted, which `release/notes.md:38` states correctly.
- Structural cause: `scripts/check_docs.py:28-35` compares README command examples only, not prose. A deliberately broken README example fails it (exit 1, good); prose changes do not.
- Reproduction: reproduced by reading and by probe. Smallest fix: correct the paragraph. Ship with finding 4.

#### 11. Public users see the switch before the release exists

- Label and severity: Design gap, Medium. Fix before release.
- Evidence: the history ticket pushes the main merge first, then the tag. The plugin marketplace and the README follow main at once, while artifacts need over two hours of qualification. If the tag run fails, they stay switched.
- Reproduction: not run. Settling test: a rehearsal timeline.
- Smallest fix: push the merge as the tag first, and fast-forward main after publication passes. No code beyond the ticket's steps.

#### 12. Two owner holds exist only in text, not in dependencies

- Label and severity: Design gap, Medium. Fix before release.
- Evidence: the 2026-10-28 decision says publication waits for the blind CLI comparison. Its ticket was withdrawn to evals, and the history ticket depends only on 0293, 0633 and 0638. The hosting-validation hold lives in progress text only.
- Smallest fix: add both as named conditions on the history ticket.

#### 13. biomcp.org stays on 0.9 after 1.0 ships

- Label and severity: Question for Ian, Medium.
- Evidence: the docs site ticket lands only after release. The site branch head edbd9f7ae dates from 2026-10-28, before typed tools. The manifest's `websiteUrl` and the package's documentation link point there.
- Options: (a) land the site in the same run, which keeps the docs true but adds one more publish step; (b) accept a short gap with a banner, which is cheapest; (c) leave it as is, which sends every 1.0 user to 0.9 docs. Recommendation: (a), or (b) if the site's rebuild would delay release. It is a publishing decision, so it stays with Ian.

#### 14. Public release assets carry internal wording

- Label and severity: Inconsistency, Medium. Fix before release.
- Evidence: generated notes are 34,161 bytes and 380 lines, with "Private no-op" once, "rehearsal" 3 times and "candidate" 7 times, from `release/workflow.py:138` and `release/notes.py:22` ("they do not describe a final release until the candidate ships"). `candidate.json` says `"publication": "requires Ian's approval before pushing the release tag"` (`release/workflow.py:187`), and `publication.github` uploads it. A 158-row schema table compares against a pre-release baseline that no 0.9 user ever had. The macOS Intel floor moved from 10.12 to 11 and no note says so.
- Reproduction: reproduced in a 0688 job that generated the notes. Smallest fix: drop the internal lines, cut the table or compare with 0.9, and add the platform floor.

#### 15. The nightly live check has failed 11 times in a row and hides its reasons

- Label and severity: Risk, Medium. After release acceptable, sooner is better.
- Evidence: of 18 runs, 14 failed, 3 passed, 1 cancelled; the last 11 failed, including j31978 at the snapshot. `scripts/nightly_live_check.py:505-526,572-591` writes reasons only to Yellow evidence. The job log shows only `make: *** [nightly-live-check] Error 1`.
- Why it matters: a check that always fails teaches people to ignore it. It also exits nonzero on drift, so genuine source changes and breakage look alike.
- Reproduction: job records read; Yellow not read. Smallest fix: print a safe count and name per source to the log, and separate drift from failure.
- Uncertainty: the failures may all be real drift.

#### 16. The gate's environment is not pinned

- Label and severity: Inconsistency, Medium.
- Evidence: commit 924a910b6 failed docs-check on Python 3.9 (j31827: `tomllib`, `extractall(filter=)`) and passed that stage on a newer Python (j31840). Also j30853. Python 3.12 is pinned in `toolchain.json` but not enforced, and 35 scripts hard-code `python3.13`. My own first job failed the same way (j32002).
- Smallest fix: one line at gate start that refuses or selects the pinned Python.

#### 17. The gate ran only on macOS arm64, and release runs a different set

- Label and severity: Risk, Medium.
- Evidence: 69 sampled gates ran on m5 or a few on Gator. Linux x86_64, Intel and Windows run only in hosted rehearsal. The release workflow runs `make fmt lint test docs-check`, not `landing-gate`, with no coverage stage. Mutation runs only in a local section.
- Smallest fix: accept this and name it, or add one Linux run to the section list. The "hosted Windows is the Windows proof" ruling covers Windows.

#### 18. Stale docs and plan text, all copies

- Label and severity: Inconsistency, Medium.
- Copies: `sdlc/planning/milestones.md:45` ("replaces" history); `docs/development.md:72` (private repository); `release/qualification.md:3,25` (pins ed9c36a as final, keeps the Windows wrapper section); `sdlc/plan.md:75` and the history ticket lines 16-17 ("operators publish", against the no-manual-pause stance); `sdlc/planning/release.md:11,13` and `upgrade.md:3` (single tool, against three tools plus typed); `release/workflow.py:138`; the 2026-10-23 clean-repo decision has no superseded marker; `docs/configuration.md:86` (noted by Reviewer B).
- Structural cause: the same truth is restated by hand across plan, docs and decisions.
- Smallest fix: one edit pass. Name the current truth once and link to it.

#### 19. Per-ticket proof sediment

- Label and severity: Risk, Medium. After release acceptable.
- Evidence: 83 of 125 Make goals are outside the core and named for single tickets. 90 of 183 scripts are referenced by nothing I could find (a lead, not proof of dead code). 63 are `check_*` scripts. 16 Rust files sit at 570 lines or more against the 600 limit.
- Smallest fix: after 1.0, sweep scripts with no caller, keeping any a decision names. The feature registration ticket handles the Rust pressure.

#### 20. Gate failures that do not explain themselves

- Label and severity: Inconsistency, Low to Medium.
- Evidence: typed per-tool caps (2000 and 3500 characters) failed 5 gates (j30728, j31078, j31317, j31728, j31840). `repository_obeys_structural_contract` failed 6 or more. `scripts/check_front_map.py:86` uses a bare `assert`, so logs show no difference (j31810, j31025). The typed description ticket's full descriptions will collide with these caps.
- Smallest fix: replace the assert with a printed diff; state the caps in the description ticket.

#### 21. Low items

- `RUSTDOCFLAGS=-D warnings cargo doc` fails at `biomcp-types` and `biomcp-adapters` (j32010, j32021): 18 bare URLs and unresolved links such as `crates/biomcp-types/src/response/error.rs:83,175`. Not in the gate. Fix or leave out; nobody relies on it.
- Release binaries keep absolute build paths: 510 matches for the build folder in the m5 binary. `release` and `.github` set no `--remap-path-prefix`. Harmless for secrecy on hosted runners, but builds differ by temp folder name.
- `pm lint`: the release rehearsal names 44 direct dependencies against a limit of five. The Pi namespace ticket has no number in its title.
- The wheel carries two copies of a 49 MB binary (31.6 MB compressed).

### Dependencies

Reviewer B owns this table. From my side only: `THIRD-PARTY-NOTICES.txt` (224,022 bytes) ships in the wheel and desktop extension, and `cargo machete` runs in the gate.

### Direct answers

- Release definition: one tag run builds a candidate, qualifies it on five platforms, stages, signs, publishes to GitHub, GHCR, PyPI, Homebrew, the installer and the MCP registry, checks each, and then promotes latest. It is clear and in code.
- Who publishes: the tag run, with no manual pause. Ian approves the tag. Docs still say "operators" in places (finding 18).
- Post-publication check: exists, but only compares the version string; no plugin or desktop extension check (finding 8).
- Upgrade notes for 0.9 users: good. `release/notes.md` covers breaking changes and links a table of every 0.9 leaf command, including retired ones such as `list`. Gaps: macOS floor, internal wording, and section names and aliases sit outside the ledger, while the stance says notes list every change.
- Tested build equals shipped build: yes for bytes. Every channel is bound to the candidate hash and revision. Publication commands themselves are untested (finding 8).
- Packed artifact contents: checked on macOS arm64 (j32009). Wheel 31.6 MB, archive 15.7 MB, desktop extension 49.4 MB, with license and notices. Private-name scan: `Yellow`, `Beelink`, `GENIE`, `Jarvis`, `biomcp-hosting` 0. `gator` 43 and `Genome` 327 were counted but not inspected; they are probably ordinary words inside strings (biology text, "navigator"), which a look at the matches would settle. Clean install worked: `version`, `--verbose`, `health --offline`, `mcp-config`, `help` all exit 0.
- Can each check fail: docs reference, workflow, schema and README example probes all failed as they should. Version-mismatch probes under `check_docs` stayed at 0 because versions come from the build, so those probes were weak; the plugin version is checked only at stage. README prose has no check.
- Do logs keep failing output: mostly yes. Gate stages print time and failure. Exceptions: nightly, `check_front_map`.
- Flakiness: of 39 failed gates in my sample, about 8 were not caused by the ticket: timing 4, Python 1, ledger 2, flake 1.

### Gate time by stage

69 sampled gate runs from the last 400 or so jobs: 30 passed, 39 failed, a 43% pass rate. Seconds from passing runs:

| Stage | Median | Max |
|---|---|---|
| pre-job | 145.8 | 834 |
| workflow-lint | 0.2 | |
| docs-check | 359.3 | 640 |
| tensor-proto-check | 5.2 | |
| audit | 0.5 | |
| secrets | 5.0 | |
| library-tests | 138.6 | |
| coverage | 433.1 | 733 |
| Total | 1079 | 2275 (min 654) |

The largest costs are coverage (cases alone 248 s) and docs-check (`test_schema_release` 143 s). Ratchets run twice and biomcp is built both plain and instrumented; removing the repeat is the cheapest saving. Coverage is 82.44% against a floor of 80.00.

### Ticket judgments

| Ticket | Judgment | Reason |
|---|---|---|
| PM count checks (0060) | Withdraw | All dates are past, lines are unmeasured, the record has no reader, and it keeps milestone 1 open |
| Rehearse the final release candidate (0293) | Needs correction | 44 dependencies fail lint; omit the ignore-ledger and load-test tickets; history wording and `release/1.0` text are stale |
| Append this repository's history (0294) | Needs correction | "Operators publish" against no pause; no owner for settings or blind comparison; tag before main; file name still says "replacement" |
| Land the docs site after the 1.0 release (0401) | Needs correction | Stale branch and 0.9 window; Ian decides (finding 13) |
| Release sections pass on main before the cut (0640) | Needs correction | Add stage, staged scan and staged Homebrew; drop the Windows VM per ruling; replace the pinned commit in `release/section.py:9`. Stays held |
| Real-time and multi-process tests (0641) | Needs correction | Remove its wait on the rehearsal; the rehearsal should wait on it; add j31824 |
| Keep Pi namespace tool discovery usable (0649) | Needs correction | Candidate reference is stale; should wait on publication or say client-only; title has no number |
| Run one hosted release section against main (0666) | Needs correction, small | Proof depends on the Markdown help fix and the held section ticket; both section runs failed |
| Moderate-risk timing tests (0670) | Needs correction | Same ordering problem as 0641 |
| Hosted schemas hide fields whose every source is off (0676) | Needs correction | Waiting on the rehearsal means a product change after the final candidate; say before the cut or after 1.0 |
| Typed tool descriptions stand alone (0679) | Needs correction | "Keeps" line contradicts the typed-tool budget ruling; per-tool caps not mentioned |
| Typed follow-ups are typed calls (0680) | Sound | Coordinate with the budget ticket, because follow-ups must name tools after grouping |
| Article text defaults to the abstract (0681) | Needs correction | Outcome and Why contradict the amendment; rewrite in place |
| Enumerated values accept any letter case (0682) | Sound | Note that client-side enum validation may still refuse lowercase |
| Typed tools fit clients that load every definition (0683) | Needs correction | Outcome superseded by its amendment; `search_gene` unsettled; budget test against "report to PM" |
| Variant lookups accept a ClinGen allele ID (0684) | Sound | |
| Feature registrations live with their owning modules (0685) | Sound | |
| Markdown help lists every flag (0686) | Needs correction | Routine gate does not run the proof; keep the failing text as the test |
| Ignore ledger binds to landed main (0687) | Sound as a stopgap | Must land first and join the rehearsal's dependencies; the cause remains |
| Pre-release whole-project review (0688) | Sound | This review |

### Strengths

- The installed gate binds every channel to the candidate hash and revision, and provenance is signed.
- Upgrade notes are honest and thorough, and a pinned 0.9.3 installer test exists.
- The ignore ledger is exact-rule, not pattern-based, and `make schema-release-check` and `make publication-check` pass.
- Rust file limit held: 0 files over 600 lines. Clippy is strict.
- Duplication is low: 15 clones, 285 lines, 0.22% of 129,008 lines.
- Ruff is nearly clean. Relative doc links all resolve.
- Case counts (420 YAML, 148 trials) match the suite ruling. Only 5 sleeps in tests.
- Gate stages print time on failure, and coverage refuses partial collection.
- The append rehearsal j30538 was clean.

### Skipped checks

- Not installed, so not run: lychee, cargo-shear, markdownlint.
- Not run: full mutation run; native Windows, Intel and Linux; hosted runs (forbidden); plugin install in a real client; any publish command; full installed gate after a help fix.
- Not read: Yellow evidence, the 12 scan findings' values, earlier reviews' findings one by one. Test sediment sampled lightly. `cargo deny` advisories left to Reviewer B.
- Clinical correctness not judged.
