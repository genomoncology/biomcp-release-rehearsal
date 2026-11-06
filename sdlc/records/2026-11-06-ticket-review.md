# Open ticket review, BioMCP 1.0, 2026-11-06

Covers all 36 tickets that were open this afternoon, judged against main at 502de0e2c and the lane heads of that time. Main moved to e196c73fe as this was finished: j32880 passed, and 0690 and 0692 landed. So 34 are open now. Lane conflicts below are rechecked against e196c73fe; everything else was read at 502de0e2c. The order follows `planning/build-plan.md`.

**How to read this.** "Read" means from reading code or recorded replies. "Run" means I ran the retained main binary (45eac64ed, the shakedown build, slightly older than main) or called a public source. No lane binary exists on this machine, so **no lane code was run**. Job results come from `pm job`. Merge results are text merges only, not compiled. Live numbers are single runs. Text marked "(Ruled; see `planning/build-plan.md`.)" in an amendment comes out when the ruling is made.

## At a glance

| Ticket | Subject | Where it stands | Left | Wave | Amendment |
|---|---|---|---|---|---|
| 0690 | Hosted server | **Landed** at e196c73fe | None | Done | No |
| 0692 | Commercial source config | **Landed** at e196c73fe | None | Done | No |
| 0710 | Study commercial flags | Routine j32870 passed, now conflicts with main in two files | S | 1 | Yes |
| 0689 | Licensed article text | No routine at head, five small bugs | M | 1 | Yes |
| 0717 | Drug names expand | Lane fails the Outcome on real data | M | 1 | Yes |
| 0691 | Whole article pools | Needs 0689 and 0717 first | S-M | 1 | Yes |
| 0718 | Source data age | j32887 passed, Open Targets date missing | S | 1 | Yes |
| 0719 | Trial interventions | Passed at head, needs main merged | S | 1 | No |
| 0720 | SEER site names | Passed, needs main merged | S | 1 | No |
| 0721 | Variant recurrence | Forms done, no frequency for a common variant | S | 1 | Yes |
| 0722 | Author search | Gated at 138c6ab18, then tooling added | S | 1 | Yes |
| 0727 | Per-request sources | Not started, on the hosting path | M | 0 and 2 | Yes |
| 0693 | Status and totals | Idle on stale history, lint failure | S | 0 and 3 | Yes |
| 0725 | No empty next page | Not started, reproduced live | S | 3 | Yes |
| 0726 | Cell line filters | Not started, source checked live | M | 0 | Yes |
| 0723 | Schemas state filters | Not started, generated check too wide | M | 3 | Yes |
| 0724 | Source text clean | Not started, one item is in the source | M | 3 | Yes |
| 0676 | Hosted schemas hide fields | Not started | M | 3 | Yes |
| 0711 | Code docs links | Not started | S-M | 3 | Yes |
| 0712 | Binaries carry no paths | Not started | S-M | 3 (early) | Yes |
| 0713 | Container pins | Not started, Proof cannot fail | S-M | 3 (early) | Yes |
| 0714 | Release profile | Not started | S-M | 3 | Yes |
| 0715 | Test sweep | Wrong scope, Proof cannot fail | M | 3 (last) | Yes |
| 0716 | Wheel, one copy | Not started, needs a decision | S or M | 3 | Yes |
| 0694 | Tool proofs in gate | Not started, may be red at first run | M | 3 | Yes |
| 0695 | Synthetic captures | Not started, case holds real data too | S-M | 3 | Yes |
| 0696 | Plugin skill expanded | Not started, fix misses marketplace users | S-M | 3 | Yes |
| 0697 | Public release text | Not started, five items | M | 3 | Yes |
| 0729 | Zero selection refused | Stalled, nearly done | S | 3 (early) | Yes |
| 0649 | Pi namespace | Harness fix not on main | S | 3 (early) | Yes |
| 0641 | Concurrency tests | Lane used up, real work not started | M-L | 3 (early) | Yes |
| 0670 | Timing tests | Not started | M | 3 | Yes |
| 0640 | Section preflight | Doc stale, preflight held for Ian | S, then L | 3 and 4 | Yes |
| 0293 | Release rehearsal | Held | L | 4 | No |
| 0294 | History replacement | Held | M | 4 | Append |
| 0401 | Docs site | Branch 704 behind | M-L | 4 | Append |

## Findings across tickets

**Problems that are gone, or never ours.**

- The EMA `?` (0724) is in the source file, so no code can fix it.
- The SEER timeouts (0720) did not reproduce.
- 0726 assumes bundled data, but Cellosaurus is a live API, so there is no store risk.
- The 12 staged artifact findings on pm's 0293 line were settled by 0665, which is on main. The pm line is stale. I did not rerun the scan.
- 0641's branch holds nothing left to land: its four commits moved to 0728. Its real work has not started.
- 0682 and 0683, named in the hosting list, are on main.
- "Bind trial history ignores" on many lanes has nothing to land: main already holds that content.
- Every other reported problem still exists on main.

**Store, namespace and pin.** Checked on all 12 lanes and on main at e196c73fe: none changes `CACHE_NAMESPACE_VERSION`, `CACHE_NAMESPACES` or the BioData pin. 0689 and 0691 touch `crates/biomcp-query/src/store/` with no table change. Hosting's snapshot stays readable (read, not run).

**Internal names on main.** 0279 has regressed. About 20 tracked files outside `sdlc/` name internal hosts or paths (a name search, so a starting list), four lanes add more, and `Makefile:391` holds a user path. `sdlc/` also goes public and has not been reviewed. No open ticket owns this, and it must clear before 0294. The 0715 amendment takes it.

**Collisions.** Across 66 lane pairs, nearly all conflicts are Makefile goals and generated files. Real code conflicts are in four places: `docs/configuration.md`, `article/mod.rs` (0689, 0691, 0717), `article/page.rs` (0689, 0717) and `drug/native.rs` (0717, 0718). Dropping per-ticket goals removes most of the rest. 0693 looks like it conflicts with everything, but that is stale history. Its five own commits apply to main with one Makefile conflict.

Against e196c73fe, text conflicts are now: 0710 in `crates/biomcp-fetch/src/lib.rs` and `docs/configuration.md`; 0718 in `placement.rs` and `architecture.md`; 0720 and 0722 in the Makefile and `docs/reference/commands.md`; 0691 and 0719 in the Makefile. 0689, 0717 and 0721 merge clean. The `lib.rs` conflict is new since the landing and I did not read it. 0690 and 0692 landed with their per-ticket Make goals (16 lines), so those are now for 0715 to sweep.

**Hidden break.** 0718 adds a field to `SourceNotice::RequestGuidance`. The merge is clean, then the build fails at new constructions in 0689 (2), 0710 (1) and 0719 (1).

**Release tooling and the rule on real hosts.** `decisions/2026-11-05-release-finds-no-bugs.md` says a ticket that changes a release section is complete only when that section passed on its real host. For 0696, 0697, 0712, 0713, 0714 and 0716, the Windows and Intel hosts exist only in the preflight, which waits for Ian, and 0640 needs those tickets landed first. See the decision in the plan.

**Proofs that cannot fail or cannot run.**

| Ticket | Proof | Problem | Fix |
|---|---|---|---|
| 0715 | Coverage floor | Floor 79.8, measured 84.54 | Compare with the commit before the sweep |
| 0715 | Mutation clause | Survivors and timeouts pass | Drop it |
| 0713 | Build from snapshot | Pins are in the normal archive too | Show sources are snapshot only |
| 0714 | Connection setup | Recorded replies open no connection | Local TLS stub, or say unmeasured |
| 0695 | "grep shows no real row" | No list to grep for | Collect the list first |
| 0697 | Installer byte check | Undefined | Compare in `published.py` |
| 0696 | Staged section | Needs five native builds | Split into unit and one-native parts |
| 0641 | Wheel run | No wheel path in the repo | Name the path, or rebuild |
| 0717 | Recorded cases | Invented drugs with `total: 1` cannot see real replies | Real-shaped fixtures |
| 0721 | Single-page cohort | Passes `all_pages_read` by construction | Multi-page cohort |
| 0723 | Generated check | Would catch at most one of the four named bugs | Cut down, move the rest to 0676 |

**Corrections to ticket text.** 0649 does gate 1.0 now, by 0293's later amendment. 0729's `tools/` is not a folder. 0697's `recover-section.yml:34` is now line 43. 0654's amendment says "after the cut", which is out of date against 0293's.

**0728's cited proof.** Its Proof names j32394 and j32395, and pm shows both cancelled with empty logs. It is complete on main. The PM should confirm what proved it.


# Bench scored build

## 0690 — Hosted server

**Landed at e196c73fe while this review ran. j32880 passed. Kept for the record.**

### Current state
j32789 routine passed at c991828e0. In native batch j32880 with 0692, running at last look.

### Detail
The problem is confirmed on main: old `connections` returns Err on accept and `?` on spawn, so one failure ends the server. Lane: `crates/biomcp/src/mcp/http/connections.rs` (`run_with`, `failure`, readers `Slots::new(max_workers)`), `logging.rs` (one line per tools/call), `http.rs`, `call.rs` (tests moved to `load_benchmark.rs`, 337 lines), `tests/runner/hosted_deadline.rs`, `http_limits.rs`, `docs/configuration.md`.

**Gate.** Routine has passed at an earlier commit. Native batch j32880 is the remaining gate. Wrap-up rule applies: RED means preserve and hand over.

### Risks and smallest derisk
- `hosted-resilience-{focused,mechanical}` goals landed with it; 0715 sweeps them.

### Size
**M.** No amendment.

## 0692 — Commercial source config

**Landed at e196c73fe while this review ran. j32880 passed. Kept for the record.**

### Current state
Accepted. j32788 routine passed at 28aa8b19e (649 cases, 84.55%). Composed head 76b744f0f is in the native batch j32880 with 0690, candidate e196c73fe, running at last look.

### Detail
Delta over 0690: registry `groups.rs` USES adds Licensed, `legacy.rs` and `legacy_restricted` removed, hotspots become Attribution "ODbL", Mutalyzer and VariantValidator terms text, README hosted paragraph replaced by a link to `docs/configuration.md#hosted-servers`, `tests declared_use/commercial.rs`, readme cases, `release/{gate,smoke,test_gate}.py`, `scripts/check_docs.py`, `release/notes.md`.

**What is left.** The batch result, then landing. Per wrap-up, on RED preserve and hand over; no repair, retry or new batch.

### Risks and smallest derisk
- Hosting impact (read in `release/notes.md` on this branch): the `restricted` disable group is retired, and `noncommercial` and `licensed` replace it. I do not know what hosting's config uses, so the PM must tell the hosting PM.
- Per-ticket `commercial-config-check` goal landed with it; 0715 sweeps it.

### Size
**M.** No amendment needed beyond the 0710 ordering below.

## 0710 — Study commercial flags

### Current state
Head 125591f77. j32857 focused and routine j32870 both passed at that head. Now that 0692 has landed, it conflicts with main in `crates/biomcp-fetch/src/lib.rs` and `docs/configuration.md`, so it needs a merge and a new routine. The flag read is done: 19 flags.

### Detail
**Flag read.** `crates/biomcp-adapters/src/cbioportal/commercial-restrictions.tsv`, 19 rows: 9 CC BY-NC-ND (asclc_msk_2024, hcc_msk_2024, msk_chord_2024, msk_ctdna_vte_2024, msk_impact_50k_2026, msk_met_2021, pancreas_msk_2024, panet_msk_2018, prostate_msk_2024) and 10 purpose limits, with id, reason and exact wording. `docs/cbioportal-study-terms.md` records the read: 2026-11-06, 551 studies, wording from `description`, with SHA-256.

**Code path.** `commercial.rs::commercial_restriction(id)` over the TSV. `crates/biomcp-query/src/study/policy.rs` (`excluded`, `notice`, `blocked`). `study/page.rs::render` filters before paging and sets `pool.size = rows.len()`. `hosted_use.rs` calls `restrict_studies()` only at Restricted.

**TCGA startup check.** No TCGA id is flagged and the test keeps `brca_tcga`. Passes by reading.

**Recurrence totals.** Recurrence is per study: `variant/recurrence.rs` calls `crate::study::blocked`, and a flagged study returns no observations. There are no cross-study totals to distort. Only `study/*` and `variant/recurrence` use cBioPortal in the query crate.

**Must 0692 and 0710 land together?** No. They need not be one commit. 0710 must land after 0692 because it carries 0692 commits (one hunk in `docs/configuration.md`), and both belong in the hosting fixes commit. No hosted refetch needed.

### Risks and smallest derisk
- **Any restricted search that drops a flagged study reports `partial` forever.** `notice(.., false)` sets `out.dropped = true` and `study/mod.rs::finish` turns Complete + dropped into partial. The test does not assert status. Recommend a policy omission counts as complete with a notice; one-line change plus one status assert. Needs a ruling (decision 4 in the plan).
- Study ids only in hosting's older snapshot have not been reviewed: one read-only comparison at hosted validation.
- Refresh of the flag list is a documented manual step: accept, no new check.

### Size
**M.**

### Amendment

```
## Amendment, 2026-11-06

A study left out by policy is not a partial answer: the reply stays complete and carries the notice. Assert the status in the declared use test. (Ruled; see `planning/build-plan.md`.)

At hosted validation, compare the hosted snapshot's study ids against the reviewed list once, read-only. Ids missing from the review are reported, not guessed.

Per-ticket Makefile goals do not land.
```

## 0689 — Licensed article text

### Current state
About 63 commits between 13:03 and 15:49, 67 files, about +2,683 lines, 32 jobs. Recent failures j32849, j32810, j32790, j32755 were all mechanical (clippy, function length, large enum variant, structural ratchet). j32726 was a real author test failure, since fixed. j32897 cancelled at wrap-up. **No full routine has run at head.** Handover says next is mechanical check at head, one final `article-rights-routine`, then a fresh review including the guard and helper delta.

### Detail, by the questions asked

**How the license lookup works.** `crates/biomcp-adapters/src/article_rights/policy.rs::licenses(leg, ids)` dedupes, takes `MAX_RECORDS = 20`, makes one batched esummary call for PMCIDs through `leg.dependency`, then calls `pmc_oa::record_license` one paper at a time, two calls per paper (LIST fresh 1 day, FILE 7 days). Budget `LOOKUP_BUDGET = 3s`, minus `FINISH_BUDGET = 200ms` from the leg's remaining time. On error it returns the partial map with no signal that anything failed.

**Time.** Cold case: up to 40 PMC OA calls. PMC_OA is `Rate::per_second(10)`, burst 1, concurrency 2, so 40 calls need at least 3.9 s, over the 3 s cap. So at most about 14 of 20 are checked for one search on its own, fewer when searches overlap. Derived from reading; **not measured**. A LitSense reply can hold 100 sentences across 65 papers (`tests/replies/article/litsense/README.md`), so papers past the first 20 distinct PMIDs never get body text, and the reason shown is `Disabled`. Warm case is SQLite only, but the filter reruns on every cached read.

**Effect on paging and pools.** The paper list, rank and totals do not change: only text is removed. But rows past the 20 cap lose their sentence in the stored pool, `items()` skips rows with no sentence, and the reason shown is the same as a real refusal. So on a cold search, later pages can show fewer sentences for a reason that is not rights. None of this runs outside `restricted`, because the filters return at once when `Use::PerRecord` is not disabled, so the bench CLI would not see this cost unless it runs at `restricted`.

**Which way it fails.** Body text fails toward no text (`permitted`: body needs `license.is_some_and(commercial)`). **Abstracts fail open**: `permitted` keeps an abstract unless the license is CcByNc, CcByNcSa, CcByNcNd or Tdm, so a lookup failure, or a paper past the 20 cap (efetch batches up to 200), keeps the abstract. That noncommercial abstract is then cached until a later read refilters it. The PM ruling m28714 (in the ticket) keeps abstracts for a record with no license, and I read that as covering this, but a failed lookup is not quite the same as no license, so a one-line PM confirmation is enough.

**Bugs found by reading (not run), all still present at b3481e7d9 by grep. I re-read (a), (b) and (c) in the code myself and they hold:**

- **(a) Permit held while waiting for another permit.** The PubMed efetch filter runs inside `exchange`, which holds an NCBI permit, and asks for esummary on the same limiter (concurrency 3). Three concurrent live efetch calls stall for 3 s, lookups fail, and abstracts pass through. The deadline bounds it, so it is not a deadlock, but under load it silently turns the check off. Fix: look licenses up before `leg.call` (ids are known) or release the permit first.
- **(b) Transient failure stored as a whole answer (conflicts with stance 1).** A failed lookup caches LitSense for 1 day with text removed, pool status reads Complete, and 0691 then publishes it as whole for 24 hours. It cannot recover: `items()` skips rows with no sentence. The reason looks the same as a real refusal. Direction is safe, the stored claim is wrong. Fix: `licenses` reports incomplete, rows say Unavailable rather than Disabled, the pool is not published, the reply is not cached (`cache_if` exists on Read; its behavior was seen, not read closely).
- **(c) LitSense refilter drops the withheld marker.** `litsense_body` retains only `pmid`, `section`, `text`. A cached `{pmid, biomcp_text_withheld}` row plus a later successful commercial lookup gives a sentence with no text and no reason. One-line fix: keep the marker.
- **(d) PubMed marker repeats.** `article_rights/pubmed.rs` inserts `<BioMcpTextWithheld/>` unconditionally, so each cached read adds another and rewrites the cache. `.child` takes the first, so nothing fails. Fix: skip if present.
- **(e) Write on read.** `store/pool.rs::rewrite_article_text` updates each row on every page read at restricted. Fix: write only when changed. This ties to W24 in 0641.

**Disk.** Raw text is never admitted at restricted: `filter_wire` runs before record and cache admission. Older unfiltered entries are cleaned on read by `filter_cached`. Entries filtered at restricted are reused at academic until their TTL ends, because mode is not checked on identity. Low impact, local only.

**Scrub remnants.** `wip/0689-withdrawn-store-scrub` (2c3e0083c) is not an ancestor of the lane. Only the existing `store clean` remains. Twelve lane commits are the withdrawn scrub and its revert. The scrub worktree and wip branch are leftovers to delete after landing.

**Route coverage.** Covered: LitSense, PubTator export and search, PubMed efetch, Europe PMC core, PMC full text (existing `pmc_oa/rights.rs`), preprint disabled path, citation evidence, stored pools, author pages, Semantic Scholar search, graph and recommend. Not read one by one: supplements and the PDF route (help text says save is refused on hosted). `PubmedBookArticle` is not filtered (low).

**Live LitSense.** No live call; stub only. This stays that way under the licensed-subset rule.

**Fixtures.** Synthetic, ids 1001-1003, stub transport. Main's LitSense captures are CC BY only (README and 13 metadata files). No licensed content found.

**Store, namespace, pin.** CACHE namespace 5/4 and fetch 1 unchanged. BioData pin unchanged. Schema change is additive and serde default. `serde_json` moved to a normal adapters dependency with a ratchet entry.

**Hosting contract.** Additive `text_withheld`, and LitSense license label is now per record. No breaking change found.

**Spiral.** Not a proof spiral in time. It is mechanical churn. But the tail from 14:00 to 15:26 is mostly Semantic Scholar citation and author paths that cannot be reached live at restricted (only stored continuations; hosted tasks start fresh). Keep what exists, stop widening.

### Risks and smallest derisk
- Bug (a) quietly disables the check under load: move lookup outside the permit.
- Bug (b) stores a wrong whole answer for a day: report incomplete, do not publish or cache.
- 3 s cap and 20 record limit may withhold too much on cold searches: one bounded PMC OA timing job, no LitSense, or accept the limit as stated.
- No routine at head: one `article-rights-routine` after the fixes above, then stop.
- `scripts/check_article_rights.sh` holds machine names and internal paths: do not land it.

### Size
**L.** Wide surface (67 files, five adapters, store, types). Remaining work is M: five small fixes, one gate, one review.

### Amendment

```
## Amendment, 2026-11-06

Before the final gate, make these changes and no others:

- Look up PubMed licenses before the source call, or release the source permit first, so a text check never waits on a permit its own call holds.
- Keep the withheld marker when a stored LitSense row is filtered again.
- Add the PubMed withheld marker only when it is absent.
- Write stored article rows only when the text changed.
- When a license lookup fails, say so with a reason distinct from a refusal, do not store the reply, and do not publish the pool as whole.

Stop widening Semantic Scholar stored routes. Keep what exists.

The per-ticket check script and Makefile goals do not land. Run the gate commands directly.
```

## 0717 — Drug names expand across trial and article search

**State.** In a lane, 20 commits ahead of main (head 49fc32160 when read). The two most recent gate jobs I saw, j32833 and j32677, failed (exit 101), before that last commit. Code, schemas, docs and cases are all written. Not landable as it stands, for the reason below.

**What the lane built (read, not run).** `drug/search_names.rs::resolve` walks MyChem, openFDA label, ChEMBL in order. The first source whose card literally matches the typed name ends the walk. If `aliases::ambiguous(card)` (`name_resolution.total != 1`) it searches the typed name only and adds a notice; otherwise it searches all of `aliases::names(card)` as one quoted OR group. Trials: `trial/drug_names.rs::expand` runs this for the free text, each `--intervention` and each `--keyword`. Articles: `article/drug_names.rs::reuse` and `article/sources.rs::drug_queries`; PubMed, Europe PMC and PubTator get the OR group, Semantic Scholar and LitSense get one request per name. `--exact-term` is added to both. All 396 lines of cases use invented drugs ("Canonical therapy") and the case file says "no clinical or live completeness claim".

**The problem still exists on main.** Ran the retained current-main Linux binary (45eac64ed), live, isolated cache, no keys: `sotorasib` 74 trials, `AMG 510`, `AMG-510`, `AMG510` all 42, `Lumakras` 16.

**The lane would not fix the report's own example (real replies run; lane code read, not run, because no lane binary exists on this machine).**

| Typed name | MyChem `name_resolution.total` | ChEMBL | What the lane does |
|---|---|---|---|
| sotorasib | 3 | 1 record, 16 names | MyChem literal match, `total != 1`, stops, searches typed term only |
| Lumakras | 2 | 1 record, 16 names | same |
| daraxonrasib | 3 | absent | same |
| AMG 510 | no match | no match | no source matches, searches typed term only, no notice |
| RMC-6236 | no match | no match | same |
| Keytruda | 1, an NDC row with 0 alternate names | 1 record, 8 names | MyChem wins with one name, no expansion |
| imatinib | 14 | 1 | stops at MyChem |
| metformin | 142 | 1 | stops at MyChem |

Sotorasib's three MyChem "candidates" are one drug: `NXQKSXLFSAEQCZ-SFHVURJKSA-N`, the same skeleton without stereo `NXQKSXLFSAEQCZ-UHFFFAOYSA-N`, and NDC product row `55513-512`. Daraxonrasib's are one InChIKey and two NDC rows. So "more than one row" is normal for a marketed drug and does not mean more than one drug. The lane's fixtures have `total: 1`, so they cannot see this.

Identifier equality alone does not settle it, though (run, `search drug` rows). `imatinib` has three different structure keys plus many product rows, and `metformin` has 12 different keys in its first 50 rows. They look like salts and combination partners; I did not check them one by one. MyChem cannot give one drug for those, so a MyChem-only rule fails, and the walk has to carry on to ChEMBL, which gave one record for both. `daraxonrasib` needs the opposite: ChEMBL lacks it, and MyChem's rows are one structure plus product rows. The two rules together cover every name in the table above: product rows with no structure key do not count, and a source with more than one structure passes to the next source.

Why `AMG 510` misses: the spaced form exists only inside one pipe-joined MyChem value, `AMG 510|AMG-510|AMG510|compound (R)-38 [PMID: 31820981]|Lumakras&reg;|Lumykras&reg;`. `DrugNameMatch::same_name` is lowercase equality. ChEMBL's lookup is `iexact` and holds `AMG-510` and `AMG510`. `search drug "AMG 510" --source mychem` does find the right two records, so the source already knows it; only our literal check drops them. That same pipe string would also be sent as one quoted search term once expansion works, so the mapper has to split it (and it overlaps 0724's `&reg;`).

**Other real-data findings (run).**
- Name counts are unbounded: aspirin 75 (ChEMBL) or 101 (MyChem), acetaminophen 110, imatinib 29. The lane has no cap.
- A 110-name OR group sent as `query.term`, the path `search trial NAME` uses, got HTTP 400 "Search error" from ClinicalTrials.gov. The same string as `query.intr` got 200. `trial.rs` maps a 400 to `UsageError::TextPositionError`, so the caller would be told their text is wrong. PubMed and Europe PMC accepted the same group.
- The right answer is reachable: `("sotorasib" OR "AMG-510" OR "AMG510" OR "Lumakras" OR "Lumykras")` returned 76 trials, against the report's 75 for all three together.
- Time cost on every search, drug or not: the three lookups run one after another before the search starts. Cold, this run: `lung cancer` 5.6 + 0.4 + 0.4 s, `BRAF melanoma` 2.3 + 0.4 + 0.6 s, `acetaminophen` 1.5 + 15.1 + 10.2 s, which is past the 20 s budget in `config.rs`. Hosted tasks start with an empty cache.
- Budget: at 4fc4941ff each term opened its own new deadline. Commit 49fc32160 fixed that, and all stages now share one command deadline. What remains: nothing is held back for the search itself, so a slow lookup can still use most of the budget and leave the real search short. Replay clocks hide this, so recorded cases cannot catch it.
- When all three lookups fail, `resolve` returns empty and adds nothing to the reply, so the search looks complete: the same silent incompleteness the ticket exists to end. In this run `get drug acetaminophen --source mychem` failed with `bad-reply`, and `get drug metformin --source openfda-label` failed with `bad-reply` (`value.unknown` on `record`) in two runs. Those two failures look like their own defect on main, not caused by this ticket. Not investigated further.

**Smaller points (read).** NCI trial search returns before expansion (`trial.rs:17`), so `--source nci` stays exact with no note. Trial page 2 reruns the lookups, because `next` carries the typed query; if the lookup fails on page 2, the page token is used with a different query. Multi-word text such as `sotorasib lung` never expands and the reply does not say so. `article/drug_names.rs::reuse` reruns the lookups each time a stored pool without a drug notice is reused, so a repeat non-drug search that was free now costs the three lookups (reply-cached where the cache holds). The pool key is safe: `exact_term` serializes only when true, so old keys are unchanged and exact and expanded searches stay separate. No store or namespace change. The new `SourceNotice::DrugNameSearch` variant regenerates every schema (190 files), which collides with 0718 and 0720 in generated files only.

**Rule check.** `decisions/2026-10-24-no-text-heuristics.md` says "no query rewriting or retries with altered queries" and also "the drug sources for drug names". 0717 is a source-driven rewrite, echoed in the reply. Stance 8 of the review stances says the PM marks the older text.

**Risks and smallest derisks.**
1. Passes its gate, fails the outcome, and the bench scores exactly this. Derisk: the amendment below, with real-shaped fixtures in the lane and one live name check on the bench binary.
2. Searches that work today start failing (budget, 400). Derisk: cap the lookup time with `deadline.child`, cap the names, fall back to the typed term with a notice.
3. Noise from weak names (`Alka rapid`, `(R)-38`, a full chemical name). Derisk: the reply lists the names and `--exact-term` exists; the cap keeps it bounded. Do not build a name quality filter. That is a word list and the no-heuristics decision forbids it.

**Size.** L as a whole; what is left after the lane's work is M: the resolver changes, the mapper split and real-shaped fixtures.

**Amendment text.**

```
## Amendment, 2026-11-06

Checked against real source replies. The Outcome stands, but synthetic cases alone cannot prove it. Real MyChem searches return several rows for one drug (`sotorasib` 3, `Lumakras` 2, `imatinib` 14, `metformin` 142): stereo forms of one structure and NDC product rows. Treating `total != 1` as ambiguous, and stopping at the first literal match, leaves those names unexpanded. `AMG 510` and `RMC-6236` match nothing, because the spaced form exists only inside one pipe-joined source value.

Changes to the build:

- Ambiguous means the candidates hold more than one structure key, not that a source returned more than one row. Product rows with no structure key, such as NDC rows, do not count. An ambiguous source, or one with no other names to give, passes to the next source and does not end the lookup. ChEMBL gave one record for `imatinib` and `metformin`, where MyChem has several structures. MyChem is the only source of `daraxonrasib`.
- Split source values that join several names, and compare names ignoring case, spaces and hyphens, only against candidates the source itself returned. This is identity comparison, not a word list.
- Cap the names in one search, configurable with a default. A capped search says how many names it left out. ClinicalTrials.gov returned 400 for a 110-name group in `query.term`.
- If the source rejects an expanded request, search the typed term and say so. Never report the caller's text as wrong.
- The lookup gets a bounded share of the time budget, so a slow lookup never costs the search.
- If every lookup fails, the reply says names were not expanded, and the result is not stored as a whole pool. A failed lookup is not the same as a name that resolves to nothing.
- Build no name quality filter. The reply lists the names searched, and `--exact-term` narrows them.

Proof adds fixtures in the real reply shape: several rows for one drug, several structures at one source, a pipe-joined value, and a code name differing only by a space or hyphen. It also adds a case where lookups use up their share of the time and the search still runs. A live check of brand, generic and code names runs on the built binary after landing, with drugs beyond the report's. A failure there is a new bug against landed work, not a landing condition.
```

## 0691 — Whole article pools

### Current state
Head 02ebf6ab5. j32875 passed focused `article_search` (9 tests), fmt, clippy and the mapping witness. Routine j32898 was cancelled at wrap-up. 20 jobs. Next per handover: review delta 3992aaf83..02ebf6ab5, then one routine gate.

### Detail
**Key change.** `crates/biomcp-query/src/article/pool.rs::key()` returns `Keys { identity, current }`. `identity` is the old hash, stored as `pool.key`. `current` is hash + `\0` + JSON of `cx.fetch.disabled_sources()`. `read()` wraps `page::read`. `complete = result.is_ok() && out.status() == Status::Complete`. Complete and key present calls `publish_current`; incomplete and no stored pool calls `unpublish_current`. `article/mod.rs`, `edges.rs` and `recommend.rs` use `store.find(&keys.current, explicit)` and `let publish = stored.is_none() && explicit.is_none();`. `store/pool.rs` moves the `pool_current` insert out of `publish`.

**Hosting store.** No table change. Hosting has 0 `pool_current` rows, so nothing there moves.

**Namespace.** Unchanged. Old local `pool_current` rows become unreachable and rebuild once, which matches the no-backwards-compatibility decision.

**`disabled_uses`** is not in the key. Fine for hosted, which sets sources. 0727 may benefit if its exclusions go through `restrict_sources` (not verified; 0727 has not started).

**Interaction with 0689.** Publication trusts `Status::Complete`. 0689 lookup failures currently read as Complete, so bug (b) above reaches here.

**Interaction with 0717.** `article/drug_names.rs::reuse(cx, query, stored, out, deadline)` drops a legacy implicit pool when names are found, straight after `store.find`. If `publish` is computed before `reuse`, rebuilt drug pools never become current and every such search silently rebuilds. Also `resolve_terms` in 0691's `mod.rs` duplicates 0717's `term::search`; keep `term::search`. By my reading of `search_names.rs`, a failed 0717 lookup does read as Complete today: the all-failed path adds no notice and no status change, so the typed-term pool would be published. `reuse` rebuilds it on the next search that finds names, so the harm is limited, and the 0717 amendment closes it.

### Risks and smallest derisk
- `publish` order with 0717: 0691 lands after 0717 and computes `publish` after `reuse`, with one test that a rebuilt drug pool becomes current.
- 0689 and 0717 failures published as whole: closed by their amendments. Add one case for each.
- `scripts/check_article_pool_mapping.py` is per-ticket and uses ssh: do not land it.

### Size
**M.** Small core, three call sites, real merge with 0689 and 0717.

### Amendment

```
## Amendment, 2026-11-06

This ticket merges after the licensed article text and search drug names tickets. Compute whether to publish after drug name reuse has run, so a rebuilt pool becomes current. Keep one term resolver, the one from search drug names.

A pool is published as whole only when no license or drug name lookup failed.

The mapping check script and per-ticket Makefile goals do not land.
```

## 0718 — Sources show how old their data is

**State.** In a lane, 24 commits ahead, level with main at b4c74dac0. j32887 passed at that head. Nearly done, with one real gap.

**Detail (read, not run).** Clean design. `Reply::data_date` and `Row::data_date` let an adapter name its dataset date. `crates/biomcp-fetch/src/data_age.rs` turns that into a notice and flags it past `source_data_max_age_days`, default 365 and configurable. `regulatory_context.rs` shows FDA events beside Open Targets phases, matches on name only, and says indications are not linked, which follows the PM's ruling. `hosted.rs::transport_next` stops `sync` being offered where it is refused.

**Store and hosting.** Safe. It adds a read of the first bundle row (`first_row`) and no table, column or key change, so hosting's snapshot stays readable. The notice gains an optional `data_date` field that old stored notices can omit.

**Gap, run live.** The report's headline source has no date. No file under `crates/biomcp-adapters/src/opentargets/` changes, and the adapter on main never asks for a version. The source does publish one: `{meta{dataVersion{year month}}}` returned year 26, month 09. So item 2 of the report gets the FDA comparison but no Open Targets date, against "every reply from a source with a known release or data date states that date". `ctgov::version` is the pattern to copy. Month precision is fine: `day()` returns None, so the date shows with no age claim. A separate cached call avoids re-keying recorded Open Targets replies.

**Other risks.**
1. Hidden merge break: adding a required field to `SourceNotice::RequestGuidance` means every new construction must set it. 0689 adds 2, 0710 adds 1, 0719 adds 1. Git will merge those cleanly and the build will then fail. Derisk: land 0718 first, or merge it with those lanes in one batch and fix them at once. It is a compile error, so it is cheap, but it should not surprise the lander.
2. `regulatory_context::attach` runs only when the sections are exactly `[indications]`, so a multi-section call shows no comparison. Acceptable. Say so if asked; no change needed.
3. It adds one Drugs@FDA call to each Open Targets indications reply. Failure is reported. Fine.
4. `scripts/check_source_age.sh` is per-ticket proof tooling. See the batch rule in the plan.

**Size.** L as a whole (198 files, mostly regenerated schemas). What is left is S.

**Amendment text.**

```
## Amendment, 2026-11-06

Open Targets publishes its data version (`meta.dataVersion`, year and month), and the adapter does not read it yet, so the source named in Why shows no date. Read it the way `ctgov::version` does, as a separate cached call, and show it in the reply. Month precision shows the date with no age claim. This does not change the PM ruling on linking indications.
```

## 0719 — Trial lists show every intervention

**State.** In a lane, 9 ahead and 11 behind main at a26fa2736. j32845 and j32866 both passed at that head. Close to done.

**Detail (read, not run).** The cap of 3 stays and `trial.rs::intervention_notices` adds one notice per trimmed row, giving the count and how to read the rest. The row already carried `interventions.total`. This meets the Outcome's second branch ("or the row states how many more"). Case covers 12 interventions, JSON and Markdown.

**Risks.**
1. The hidden drug stays hidden. The report's complaint was that zoldonrasib did not appear; the reply now says "1 more", and the reader must make another call. That is what the Outcome allows, and the size limit is kept.
2. The notice sits on the source, not the row, and hard-codes the CLI wording `biomcp get trial NCT... arms`, which a hosted typed-tool caller cannot type. Small wording fix if the lane touches it again; not worth a new gate run on its own.
3. Needs `data_date: None` once 0718 lands (see 0718).

**Size.** S.

**Amendment.** None needed.

## 0720 — SEER answers by site name

**State.** In a lane, 18 ahead and 11 behind main at 557df8f35. j32830 passed (the long run) and j32756 passed. Needs a merge of main and acceptance.

**Detail (read, not run).** Site names resolve through the catalog, several missing options come back in one error (`--dataset, --period`), period and stage codes carry labels, an ambiguous name returns both sites, and a slow reply reports `deadline-exceeded`, never a zero. The `value.unknown` membership and method problems are handled.

**Risks.**
1. All cases are "Synthetic publisher catalog replay; no provider requests", the same blind spot as 0717. The real catalog has 73 sites, some with overlapping names. Derisk: one live call in the bench smoke list (below): pancreas by name, then incidence and survival.
2. `scripts/seer_live_probe.py` hard-codes site 40, the report's example, and writes to `dist/`. Its job was done by the amendment. It should not land.
3. Timeouts stay unexplained, which the amendment already says. Nothing to do unless they recur.

**Size.** M, done.

**Amendment.** None needed.

## 0721 — Variant recurrence accepts what it says it accepts

**State.** In a lane, level with main at d924ccd0d. j32874 passed. Forms work is done. One half of the Outcome is not.

**Detail, run on the shakedown binary.** `variant recurrence` with study `msk_impact_2017`: coordinates, CAid, reference HGVS, `clinvar:N` and VCV all return `complete`. Two forms fail. `chr12:g.25245350C>A` is listed but refused with no `--build`; the lane fixes that with `identity_build` (read). `rs121913530` fails because the rsID has several alleles; the lane's new wording says why (read). The tester's likely inputs also included `KRAS G12C`, which stays refused; the lane does not list it, which meets "Forms it cannot use are not listed".

**Gap: no frequency comes back for a common variant (run).** The command reads one page and counts within it. With the default limit 10: `observed_sample_count` 1, `all_pages_read` false. With the maximum, 100: 22, still `more: true`. KRAS in this study has 1,670 rows, so a caller needs 17 calls and must add them up. The lane's proof uses a single-page synthetic cohort, so `all_pages_read: true` passes there. The 100 is our own cap (`query/recurrence.rs`, default 10). cBioPortal returned all 1,670 rows in one request in 0.3 s (1.1 MB). The reply holds counts, not rows, so a bigger page costs the caller nothing. Smallest fix: raise the default and maximum `limit` to 5,000, about 3.3 MB by that ratio and inside the 8 MiB body limit, which is a change to two constants plus regenerated schemas. The request key changes, so old cached pages simply go unused. That is not a namespace change.

**Second defect (read, checked in both lanes).** `recurrence.rs:92` names `Source::CancerHotspots` for the page step, on main and in this lane. `next::finish` drops a step whose source is disabled, so the page step vanishes wherever Cancer Hotspots is off, including under 0727's per-user exclusions. The status and totals lane already has the one-word fix, but that lane lands after the bench build and possibly after 0727. Make the same change here; identical changes merge cleanly.

**Hosting.** Recurrence calls the live cBioPortal API (`cohort`, `mutations`), not the store, so it does not depend on hosting's snapshot (read). 0710's study flags still apply, and 0710 is where that lives.

**Decision needed.** "Frequency": `decisions/2026-10-24-variant-slice-rulings.md` rule 5 says the command "shows the denominator", while the option help says the selected size "is never an allele denominator". Recommendation: report the observed sample count and the selected sample count after reading all pages, and compute no percentage.

**Size.** M, done for forms. The gap is S.

**Amendment text.**

```
## Amendment, 2026-11-06

Checked against the public API. A common variant cannot finish today: reading a real KRAS cohort takes 17 calls at our cap of 100 rows, while the source returned all 1,670 rows in one 0.3-second request. The reply holds counts, not rows, so a larger page costs the caller nothing. Raise the default and maximum `limit` to 5,000, so a common variant reads every page in one call and reports `all_pages_read: true`. Paging stays for anything larger.

"Cohort mutation frequency" means the observed sample count beside the selected sample count once all pages are read. BioMCP computes no percentage, because the selected sample list is not always the set assayed for that gene.

The page step names Cancer Hotspots as its source. It must name cBioPortal, or the step disappears wherever Cancer Hotspots is off. Ticket 0693 holds the same one-word fix; make it here too.

Proof adds a multi-page cohort.
```

## 0722 — Author search works from open sources

**State.** In a lane. Done and gated: j32772 passed at 138c6ab18 on gator, 650 cases, coverage 84.54%, all stages green. Everything after that (f35854ff1, 15f55cbf7, 93aa0d523, bc8e12c3b) only adds `scripts/check_author_artifact.py` and a Makefile goal. Its jobs j32858 and j32879 failed (exit 2) and j32903 was cancelled. So a finished ticket is now spending jobs on a checker of its own artifact.

**Detail.** On the shakedown binary with no key, `search author "KRAS G12C"` returned 0 rows from Semantic Scholar, `status: complete`, so the reported problem exists and looks like an empty answer. The lane (read, not run) adds `AuthorSearchSource::Auto` and `Pubmed`, and `author/literature.rs` runs a plain PubMed search and an esummary. It returns papers with their author names and a fixed sentence saying identities are not resolved. Paging is safe: `retmax` is 9999 and the cap matches. No text beyond citation data, so 0689 has nothing to filter here (read).

**Risks.**
1. Process: the review script must not land. It hard-codes this machine's job folder, which holds the private repo name, plus a host name, a commit and a hash. That breaks the public text rule and is the proof spiral's strongest sign (a tool that checks a check). Derisk: reset the lane to 138c6ab18 plus a merge of main, and rerun the gate once.
2. Product: a pharma tester asking "who leads KRAS G12C" gets 5 papers by default and has to tally names by hand. Nothing is ranked, and there are no affiliations or ORCIDs. The ticket's Proof is met, but the workflow row may only move from Broken to Weak. Europe PMC is named in the Outcome and the lane does not use it; its metadata may carry ORCIDs and affiliations (not checked). Derisk: land as is, and let the bench say whether more is needed. See the decision in the build plan.
3. 0723 overlap: the lane keeps advertising `semanticscholar`. 0723 fixes that after this lands.

**Size.** M, done. What is left is S: drop the script, merge, one gate.

**Amendment text.**

```
## Amendment, 2026-11-06

The product work is done and passed the routine gate. A script that re-checks its own build artifact does not land: no review tooling, host paths or pinned hashes in the repo. Land from the gated commit with main merged in.

Europe PMC stays out of this landing. If the evaluation shows that a list of papers with author names does not answer the key opinion leader workflow, affiliations and ORCIDs from Europe PMC get their own ticket.
```

# Hosting fixes commit (beyond the tickets above)

## 0727 — Hosted callers narrow sources per request

**State.** Not started, no lane. It is in the hosting fixes commit, so it is on the critical path and needs a lane now.

**Detail (read, not run).** Smaller than it looks. `Fetch::restrict_sources` already adds exclusions and cannot remove them ("Existing source restrictions cannot be removed"), so "can only narrow" holds by construction. The path: `http.rs` reads `x-biomcp-use` and key headers per request, `HostedCall` carries them to the worker, `hosted_use::restrict` applies them. The new header follows the same route: parse in `wire.rs`, carry on `HostedCall`, union in `hosted_use::restrict`.

**Does the store obey it?**
- Source snapshots: yes, `source_snapshot.rs` passes `cx.fetch.disabled` on every read, so 19 modules are covered.
- Article pools: only after 0691, whose new key hashes `cx.fetch.disabled_sources()`, so each exclusion list gets its own pool. Depends on 0691.
- Reply cache: excluded sources make no call (read in `allowed_fields` and the leg path; not traced exhaustively).
- 0717 already withholds stored expansion names when their source is off.

**Risks.**
1. Collision with 0690, gone: it landed, so branch from main at e196c73fe or later.
2. Follow-ups could leak or lose the list: `command/steps.rs` writes `--disable-sources` into next steps from globals. A header list must not be echoed as a flag; the wrapper resends the header. Case needed.
3. `catalog::excluded`: feed the header through it, or tools, help and `next` disagree with the reply.
4. Hosting's contract: `x-biomcp-exclude-sources` and "strips client copies" are agreed. Open points: are group names like `noncommercial` accepted, or only source names; are components like `drugbank-open` leavable; which form does the list take (CLI JSON, an MCP tool or an endpoint). Recommendation: source names only, and a CLI JSON list in the style of `biomcp key list`, because the wrapper reads it at build time. Confirm through the PM, since hosting is PM-only.
5. An unknown name is a refusal for that user's every call until settings change, so a renamed source could lock a user out after an upgrade. The contract says reject; keep it, and the catalog is the guard. Mention it to hosting.
6. Key expectation: NCI is `Open` with a required key, so a user's key works at `restricted`. OncoKB and AlphaGenome are `Licensed`, so they stay off at `restricted` even with a user's key, as the 2026-11-04 ruling says. Ian's words on 2026-11-06 ("support API keys") may expect more. This does not block 0727, but the PM should confirm it.

**Size.** M: one header, one catalog, a clear refusal, and cases. The seam exists.

**Amendment text.**

```
## Amendment, 2026-11-06

The seam exists: `Fetch::restrict_sources` only adds exclusions, so a request cannot widen what the deployment serves. Parse `x-biomcp-exclude-sources` beside the other per-request headers, carry it on the hosted call, and apply it where `hosted_use::restrict` applies the use limit. Feed the same list to `catalog::excluded`, so the tool list, help and `next` agree with the reply.

Names are source names only, not groups. The list of names users may leave out is CLI JSON in the style of `biomcp key list`.

Stored results: source snapshots already check disabled sources on read. Article pools need 0691's key, so this lands after 0691. Follow-up steps must not turn the header into a `--disable-sources` flag.

Proof adds a case where two callers with different lists run the same article search and neither receives the other's stored pool.
```

# Product work after the scored build

## 0693 — Status and totals tell the truth

### Current state
Idle since 13:26. Last job j32481 at 22369f5fb failed on lint: `crates/biomcp-types/src/adverse_event/query.rs:124` `try_from` is 85 lines, limit 80 (seen in job log). pm shows only "started".

### Detail
Own commits: df582d277, 9451f479b, d6ce20909, 21cb38556, 22369f5fb.

Main still has the reported problems (read): `trial.rs:93` `status::search_for`, `protein/sections.rs` takes n with no count, `drug/native.rs` `received` counts mapped items, `bundle/store.rs` strict validate.

Every Outcome item has code and a case in the lane. Not gated. The table pairs cases to items by their names; I did not trace each case through its code.

| Item | Code | Case |
|---|---|---|
| Trial metadata failure | `trial.rs` drops `status::search_for`, uses `status::item` | trial/metadata-failure |
| FAERS offset cap | `adverse_event/query.rs` `offset > 25_000` to `ErrorMessage::FaersOffset` | adverse-event/faers-offset-cap |
| Exact application total | `openfda_label/search.rs` queries `openfda.application_number.exact` instead of filtering after paging | drug/exact-application-total |
| Exhausted or refused offsets | `openfda.rs::page` probes skip 0 limit 1, used by 7 adapters; `offset < total` with an absent page is `bad_reply` | drug/exhausted-label, drug/native-refused-offset |
| Native received count | `drug/native.rs` `page.received()`, `opentargets/map.rs::Search::received` | drug/native-refused-offset |
| Protein card counts | `protein/sections.rs` `bounded`, `continuations`, `ProteinSectionPage { items, total }` | protein/bounded-card-counts |
| Bundle relocate, restart and recurrence paging | `bundle/store.rs::Part::relocate`; `variant/recurrence.rs` Page step source to Cbioportal | pathway/wiki-restart-policy, variant/recurrence-page-policy |

**Overlap.** 0725: 0693 does not touch `next.rs`, so they stay separate; start 0725 after 0693 lands. 0718: resolved cleanly in simulation. The protein item needed 0680, which has landed.

### Risks and smallest derisk
- Looks like it conflicts with every lane (`http.rs`, `typed.rs`, `typed/followups.rs`, `summary.rs`, others). All of that is stale 0680 history. Derisk: cherry-pick the five own commits onto main. Done in a scratch clone: only the Makefile conflicts, 31 files +633/-190 excluding replies, and the result merges clean after all other lanes in all three simulated orders.
- Lint failure: split `try_from`.
- `openfda.rs::page` changes 7 adapters at once: its behavior has cases for labels and native drugs only. Device and recall paths rely on the shared code (not read per adapter).

### Size
**M** overall. **S** left once rebuilt: one lint fix, one gate.

### Amendment

```
## Amendment, 2026-11-06

Rebuild this branch from main by applying only this ticket's five commits. The older follow-up history already landed. Drop its Makefile goals, and do not add per-ticket goals. Split the adverse event query conversion to pass the length lint, then run one routine gate.

Next-command work stays in its own ticket, which starts after this one lands.
```

## 0725 — Paging never offers an empty page

**Current state.** Not started. No lane, no branch.

**Detail (read, and the bug reproduced live).**

- Reproduced on the shakedown binary: `search trial --sponsor "Revolution Medicines" --limit 20` reports total 20 and offers a next page, and that page returns 0 rows.

- The reported bug still exists on main, and the cause is one missing check. `crates/biomcp-query/src/trial.rs:142`, in `apply_page`, offers the next page whenever ClinicalTrials.gov sends a `nextPageToken`. It never compares against `page.total`, although the total is known and is written to `outcome.page` just above it (`:135`).
- The source really does send a token at the exact end. In `tests/replies/trial/country/`, a capture with `pageSize=1` holds 1 study, `totalCount` 1 and a `nextPageToken`. The README there says one of the three such files is the unmodified capture and the other two reuse it as stubs. So this rests on one real capture, not three. Across the recorded trial replies, no short page carries a token, which fits "a full page gets a token even when nothing follows" and matches the report: limit 20, total 20. Two recordings show a full page with no token (`trial/name-fallback`, `trial/gene-notices`, both `pageSize=1`). I did not trace them, so that rule is likely, not proven. The fix does not depend on it.
- `next.rs:94` is not the cause. `has_more` there only decides network steps. It never removes a page step.
- `window.has_more()` (`crates/biomcp-adapters/src/native_window.rs:85`) is correct when a total exists. With no total it returns `received == limit`, so a full last window offers a page that comes back empty.
- I classified all 87 `StepKind::Page` sites by their gate. Nearly all already check the total (for example `trial/nci.rs:135`, `adverse_event.rs:119`, `diagnostic/device_search.rs:40`, `organization.rs:46`). The one known-total family with no check is the ClinicalTrials.gov search in `trial.rs`.
- Unknown-total families are a different case, not this bug. `cell_line/mod.rs:49` offers a page on a full window, on purpose: `planning/slices/cell-line.md` records that the JSON has no total and that "No command fetches an extra page". Windows with no total behave the same way through `has_more()`.

**Risks and the smallest derisk.**

| Risk | Smallest derisk |
|---|---|
| The Outcome's first sentence, "only when one exists", cannot be met where the source gives no total without an extra fetch, which the cell-line slice forbids. A lane could spiral on it. | Amendment below: the promise applies to known totals, and an unknown total keeps today's behavior. |
| The Proof asks for three cases "for each paging family". That is up to 87 sites x 3 for one line of behavior, a proof spiral. | Cases per mechanism, not per family: source token (trial), known-total window, stored pool, unknown total. |
| A global guard in `next::finish` looks tempting, but `result.page` can describe an outer list while a page step belongs to a nested section (`Navigation::Nested`, card sections). It could remove valid section steps. | Fix `trial.rs` only. Do not add a global guard. |
| A token page can arrive with no total. `ctgov/mod.rs:176-186` then makes a count call, which can fail and leave the total unknown. A plain `total <= offset + returned` check on an unwrapped value would hide real pages or panic. | Keep the token when the total is unknown. Include this in the unknown-total case. |
| `trial.rs` is being edited by 0693 (removes `status::search_for`, line 93), 0717 (`drug_names::expand`, passes `expanded` to ctgov) and 0719 (`intervention_notices`). None touch lines 135-152, so a textual conflict is unlikely, but 0717 changes the search the token belongs to. | Start 0725 after 0717 lands, and run the end-of-results case with an expanded drug name. |

**Overlap with 0693.** The two are complementary, not duplicates. 0693 changes what happens when a caller is already past the end on openFDA (new `openfda::page` in `crates/biomcp-adapters/src/openfda.rs`, adapter files only). 0725 stops the offer on ClinicalTrials.gov. They share no openFDA code, so the ticket's "Works with the end-of-results fix in 0693" creates no ordering need beyond the shared file above.

**Size: S.** A one-line gate plus four cases. It only grows if the per-family Proof stands.

**Amendment text.**

```
## Amendment, 2026-11-06

Reproduced live: a search with exactly 20 results offers a next page, and that page is empty. `trial.rs` `apply_page` offers the next page on the source's page token alone and never compares the known total. A recorded ClinicalTrials.gov reply shows a token on a full last page. Other known-total families already check the total.

The Outcome's promise covers known totals. Where a source gives no total, a full window may still offer a next page, because no command fetches an extra page to find out. That page comes back empty and says so.

Fix `apply_page`, keeping the token when the total is unknown. Do not add a global guard in `next::finish`, because a reply's page can describe an outer list while a step pages a section.

The Proof's cases run per mechanism, not per family: a source token, a known-total window, a stored pool and an unknown total. Each has a case at exactly the total and one row short. Land after 0717, and include a drug name that expands.
```

## 0726 — Cell line search ranks and filters for research

**Current state.** Not started. No lane, no branch.

**Detail (code read, source checked live).**

- There is no bundled Cellosaurus data. It is a live API: `cellosaurus/decl.rs` has `bundles: &[]`, and `SPEC.base` is `https://api.cellosaurus.org`. The ticket's "If the bundled data lacks sequence variations" rests on a false premise. So this ticket cannot change the store format, does not touch hosting's snapshot and needs no refetch. It does change public schemas.
- Sequence variations are mapped, but only on cards. Search rows do not carry them: `SEARCH_FIELDS` is `ac,id,sy,ox,di,ca` (`cellosaurus/mod.rs:32`).
- Parent or derived status is not read anywhere on main.
- Search is one native window in source relevance order (`cellosaurus::search`, `cell_line/mod.rs:18`).
- **Live check (run, a few calls on the public API).** The source can filter both things. `id:AsPC-1 AND NOT hi:*` returned only the parent line. `var:KRAS AND var:"p.Gly12Asp" AND ox:9606 AND NOT hi:*` returned 163 rows, and all 163 really hold KRAS p.Gly12Asp on one variation. The `hi` field comes back as `derived-from`. `var:KRAS` alone also matches lines marked `None_reported`. Sorting was not tested, because the filter is enough.

**Risks and the smallest derisk.**

| Risk | Smallest derisk |
|---|---|
| Ranking inside a window cannot work: the slice records `NB4` with derived lines filling the first window. | Source-side filter, now confirmed live. No local pool. |
| Default ranking conflicts with `decisions/2026-10-24-no-text-heuristics.md`, which rules out rerankers and allows "a sort the agent asks for". 0617 says free text alone stays byte-identical. | Decision below. |
| Silent incompleteness, the harm the report is about. Mutation text varies, `G12D` is stored as `p.Gly12Asp`, and the query does not tie gene and mutation to one variation. The 163-row sample had no false matches, but one sample does not show there are none. | The reply states the match is on source text. Conversion from `G12D` is allowed only as a typed parse of a marked option. |
| A gene filter alone returns lines without the mutation, pushing callers to one card call per row. | Add the variation field to filtered searches only, and check `tests/cell-line-command-budgets.yaml`. |
| Adding to `SEARCH_FIELDS` changes the request, so all 5 recorded search replies stop matching. | Request new fields only when a new filter is used. Unfiltered replies stay byte-identical. |
| `CellLineSearchCommand::unfiltered()` checks only species and disease. A new filter not added there would lose its echo and paging. | Add new filters there, and page a filtered search in a case. |
| `Changes` omits generated help, `docs/reference/commands.md`, `tests/replies/cell-line/` and the budgets file. | Named in the amendment. |

**Should it split?** No. The live check removed the reason: both halves work the same way at the source. The earlier advice to split was written before that check.

**Decision needed (PM, or Ian if the PM thinks it touches the no-rerankers ruling).** Default order. A: keep the source's order and add a caller-chosen derived filter. It fits the standing decision, and agents find it through help and `next`. B: parents first by default, which answers the report directly but needs an exception and changes current replies. Recommendation: A.

**Size: M.** No store work.

**Amendment text.**

```
## Amendment, 2026-11-06

Cellosaurus is read from its live API and has no bundle, so this ticket changes no store format and needs no refetch. Replace "the bundled data" in the Proof with "the source".

Checked live. The source can filter both things this ticket needs. `id:AsPC-1 AND NOT hi:*` returned only the parent line, and `var:KRAS AND var:"p.Gly12Asp" AND ox:9606 AND NOT hi:*` returned 163 lines, all holding that variant. The `hi` field names a line's parent, and no reply on main reads it yet. Card replies carry variations; search rows do not.

Filters go in the source query, as with species and disease, and never after fetching, so paging stays correct. Re-sorting a returned window does not meet the Outcome: the exact NB4 line is outside its first ten rows. If a filter does not work at the source, that part stops and goes to the PM. Do not build a local pool.

`var:KRAS` alone also matches lines the source marks as having no reported mutation, so a gene filter alone must say what it matched. Mutation text is matched as the source writes it. Converting `G12D` to `p.Gly12Asp` is allowed only as a typed parse of a marked option. The reply says the match is on source text and that gene and mutation are not tied to one variation, so a short result is not read as complete.

Request the new fields only when a new filter is used, so an unfiltered search stays byte-identical. Add each new filter to `unfiltered()`.

Changes also covers generated help, `docs/reference/commands.md`, `tests/replies/cell-line/` and `tests/cell-line-command-budgets.yaml`.

The default order stays the source's, and callers choose the derived filter, because `decisions/2026-10-24-no-text-heuristics.md` rules out rerankers and allows a sort the caller asks for. (Ruled; see `planning/build-plan.md`.)
```

## 0723 — Schemas state each filter truthfully

**Current state.** Not started. No lane, no branch. All four named problems still exist on main, by reading. One has a different cause from the report's (the hosted filter, not a schema typo).

**Detail (read, not run).**

- **`product_code` for 510(k): still refused.** `crates/biomcp-types/src/query/diagnostic/search_validation.rs`, `validate_device`, refuses it for `fda-pma` and `fda-510k`, because `mod.rs:122-123` counts it as a recall filter. The schema text does say recall only (`mod.rs:83`, "Exact native FDA device recall product code"), so the refusal is consistent with the long text but not with what callers need.
- The ticket's note that `fda_510k/` "already reads `product_code`" is misleading. `fda_510k/raw.rs:14` reads it from replies. There is no search filter: `device_search.rs::expression` builds only name, applicant and decision date. Only `fda_device_recall/mod.rs:47` sends it. So this is a new filter in `expression` plus a validation change, and it reaches PMA too, since the function is shared.
- **NCI disease input: still mismatched, but the error is honest.** The field's first line is "Disease or NCIt." (`query/trial.rs:50`). `nci_disease_invalid` (`:562`) refuses anything but `C` plus digits before any source call. The error already says "NCI disease filters require an NCIt identifier such as C3224; use --keyword for free text". `--condition-id` already resolves MONDO and DOID to NCIt through MyDisease. There is no name resolver.
- **Open Targets section error: still there, and it is wider than Open Targets.** `DrugGet::supports_sections` (`drug/query.rs:335`) requires at most one section for every drug source. When it fails, `prepare.rs:160` lists all of that source's sections as valid values, without saying one at a time. The schema's `section` is an array with no limit, so every source with two or more sections has the same gap.
- **`semanticscholar` vs `semantic-scholar`: the true cause is the hosted filter.** Main's schema lists both (`author/query.rs:11`). `catalog.rs::filter` drops enum words that parse to an excluded `Source`, and it keeps any word that does not parse. `semantic-scholar` parses, so it is removed. The alias `semanticscholar` does not parse (`canonical_enum!` matches canonical names only), so it stays. That explains why the tester saw only `semanticscholar` on hosting. It is the only source alias: `alias =` appears once in `biomcp-types/src`.
- 0722's lane keeps the alias in its new list (`semantic-scholar`, `semanticscholar`, `pubmed`, `auto`), so hosting would still offer a spelling that leads to a source that is off.

**The generated check: what it really needs.**

- One place lists everything: `biomcp_types::command_paths()` plus each command's schema. `request.json` holds 61 commands, 481 fields, 82 enum fields and 566 enum values, 226 of them `country` (counted by script from the schema).
- A generated walker already exists: `crates/biomcp/tests/runner/enum_case.rs` (`query-validation/enumerated-letter-case`). It walks every command, sends one value per enum field through the CLI and the typed tool with `--offline`, and fails if the value does not reach an accepted query. A hand table, `context()`, supplies the prerequisites for each special case.
- **It stops at argument checking. It never reaches an adapter.** Nothing offline can show that a filter is really sent. Only a recorded case shows that.
- **As written, the check would catch one of the four named bugs at most.** `product_code` is plain text, not an enum. Its rule is "this filter with this source", and the schema does not hold that fact. It exists only in hand-written validators and doc text. The Open Targets rule is a count of sections. The NCI rule is a value format that depends on the source. The alias problem appears only in the hosted list, and the walker reads the local one.
- The enum half also mostly cannot fail. Most schema enums are generated from the same Rust enum that accepts them. Only hand-written lists (`extend("enum"...)`, about six sites) and `x-enum-by-field` can drift.
- "Every filter a command advertises is accepted" needs a fact that does not exist yet: which sources answer which field. That is exactly what 0676 builds. Building it here would be this ticket's second new mechanism and would duplicate 0676.

**Risks and the smallest derisk.**

| Risk | Smallest derisk |
|---|---|
| The generated check spirals into a field-by-source matrix with a hand table for each rule. | Cut it down, as below. Extend the existing walker and build no new one. The filter-by-source check moves to 0676. |
| Extending the walker to all 566 values multiplies process launches. It runs several per value, so `country` alone adds 226 x several. | Canonical spelling only for added values, and sample `country`. Record the added gate time. |
| The Open Targets fix looks like one source but touches every `get drug` caller. A `maxItems: 1` changes the schema for all sources. | Fix the message and state one section per call in the schema, for all sources, and say so in the ticket. |
| Removing the alias from the advertised list could break `tests/cases/author/source-alias.yaml`. | Stop advertising it and keep accepting it. |
| File collisions: `author/query.rs` (0722), `prepare.rs` (0720), `device_search.rs` (0693 rewrites its call), `query/trial.rs` (0717). Generated `request.json` and `docs/reference/commands.md` collide with nearly every lane. | Start after the bench scored build lands. Regenerate rather than hand-merge. |
| `product_code` needs a new recorded reply, because a new request means a new key. | openFDA is public, so a small capture or a stub is fine. Budget one job. |
| New text and options count against the 0683 tool budget. | The existing size test covers this. No new step. |

**Decisions needed (PM).**

| Decision | Options | Recommendation |
|---|---|---|
| The generated check | A. As written, here. B. Walker extended here, filter-by-source moved to 0676. | B. A cannot be built without 0676's facts. |
| NCI disease input | State it, or resolve names. | State it. Resolving a name needs a resolver that does not exist, and `--condition-id` already resolves CURIEs. NCI stays off on hosting until a caller brings a key, so resolving buys little now. |

**Size: M.** Four small fixes plus a walker change. It becomes L if the generated check stands as written.

**Amendment text.**

```
## Amendment, 2026-11-06

Read in code, not run. All four items still exist on main.

`product_code`: `fda_510k/` reads this field from replies, not as a search filter. Add it to `device_search.rs` `expression`, which PMA shares, and stop `validate_device` refusing it.

Author source names: the hosted schema filter keeps any word that is not a source name, so it removes `semantic-scholar` and leaves the alias `semanticscholar`. Stop advertising the alias, keep accepting it, and check the hosted tool list. Land after 0722, which edits the same list.

Open Targets: `get drug` takes one section per call for every source, and the error lists all of the source's sections without saying so. State the limit in the schema and in the error, for every source.

NCI: state the input and do not resolve names. The field says an NCIt code is needed for NCI, and points to `--condition-id` for MONDO or DOID identifiers.

The generated check is cut down. Extend the existing `query-validation/enumerated-letter-case` walker from one value per enum field to every value, sampling `country`, and add a pass over the hosted tool list. Build no new checker. It covers argument checking only. It does not reach adapters, so each named filter still has its own recorded case.

"Every filter a command advertises is accepted by that command" moves to 0676, because it needs the per-field source facts that ticket adds.

Land after the bench scored build, because this ticket shares files with 0693, 0717, 0720 and 0722.
```

## 0724 — Source text arrives clean

**Current state.** Not started. No lane, no branch. Four of the five items still exist on main. The EMA `?` is in the source, so it needs no code.

**Detail (read, not run, apart from reading data files).**

| Item | On main? | Evidence and cause |
|---|---|---|
| EMA `&nbsp;` | Yes | A local copy of EMA's `medicines.json` (dated 2026-10-26, 2,754 records) has 977 `&nbsp;`, 121 `&lt;` and 39 `&gt;`, and no other entity. `ema/map.rs:81` passes `therapeutic_indication` through unchanged. |
| EMA `?` for `≥` | **In the source** | The same file is valid UTF-8 with no replacement characters, and 69 indications read like "haemoglobin concentration of ?10 g/dl". 92 real `≥` sit in the same file. Our decode cannot make a `?` out of valid UTF-8. Caveat: this is an older local copy, not hosting's snapshot, and I assumed it is unmodified publisher bytes. |
| Trial `\<`, `\^2` | Yes | In the source: recorded ClinicalTrials.gov replies carry them in the JSON values (43 `\<`, 44 `\^`, 89 others such as `\[CR\]`). Nothing removes them. The adapter sends no `markupFormat` parameter. |
| FAERS codes | Yes | `detail_map.rs:105` copies `reactionoutcome` unchanged, and nothing on main labels any FAERS code (`patientsex`, `drugcharacterization`, `qualification` are bare too). |
| GWAS 0.0 | Yes | `gwas/map.rs:27` copies `p_value` beside `exact_p_value`. Sort and threshold logic already use the exact value. No fixture I scanned has this case. |

**Is there a shared text cleaner?** No. `native_text.rs` and `Text::from_source` only admit text and escape control characters. `Text::from_source` lives in BioData, which is pinned and must not change here. The new code belongs in `biomcp-adapters`.

**Store format.** No risk, if the fix stays at read time. EMA is a stored bundle, but cleaning in `ema/map.rs` leaves stored rows, the word index (`raw.rs` `keys`, names only) and hosting's snapshot unchanged. ClinicalTrials.gov, FAERS and GWAS are live replies, and the cached raw reply stays as it is. The fixes change replies, and FAERS and GWAS change reply schemas. Do not clean inside `Row::keys` or at sync.

**Risks and the smallest derisk.**

| Risk | Smallest derisk |
|---|---|
| **"One shared text cleaner" read as one cleaner for all sources.** Put in `NativeText`, it would reach 51 adapter files. Backslashes are markup only in ClinicalTrials.gov, and entities only where a source double-encodes. XML sources are already decoded by their parser, so decoding again could change meaning. | One shared module with two named functions, called only for EMA text and ClinicalTrials.gov text. Never in `NativeText`. |
| **Trial results skip the mapping step.** They are the exact fields the report named, and they deserialize straight into public types (`TrialPostedOutcome`, `TrialEvent`; 57 `Box<str>` fields in `trial_results.rs`) with no adapter map. A cleaner on `problems.field` never reaches them. | Name this in the ticket. Clean them in `ctgov/detail/results.rs` before they become public values, not in `biomcp-types`. |
| **The Markdown view adds its own escapes, on purpose.** `view/escape.rs` (0348) escapes `\`, `*`, `[`, `]`, `#` and more, and writes `<` as `&lt;`. After the fix, raw Markdown still shows `\[CR\]` and `&lt;`. A reviewer could read that as a failed fix, and removing it would undo a security control. | The Proof checks JSON for clean text. Markdown keeps 0348's escapes. Stated in the amendment. |
| The Outcome says damaged characters are "repaired at their source or marked". The `?` cannot be spotted without guessing from text, which `decisions/2026-10-24-no-text-heuristics.md` forbids. | Leave the value. Say so in the ticket, and add one line to `docs/cautions.md`. |
| Asking the source for a different markup format might avoid unescaping, but it changes every recorded ClinicalTrials.gov request. I did not check whether it would work. | Unescape in the adapter instead. |
| "FAERS codes" is open-ended. The route code list alone is long. Hand-made lists are forbidden, but a source's own lists are allowed. | Decision below. Copy labels from openFDA's published field reference, which `decisions/2026-10-29-source-number-units.md` already cites. |
| GWAS: blindly dropping `p_value` loses a real value. | Omit it only when it is zero and `exact_p_value` is present. A synthetic reply is needed, because I found no fixture for this. |
| 0718 is editing `ema/raw.rs`, `faers/counts.rs` and `ctgov/search_query.rs`, and it budgets EMA replies. | Start after 0718 lands. |

**Decision needed (PM).** Which FAERS codes get words?

| Option | Trade-off |
|---|---|
| A. Reaction outcome only. | Least code, but the same bare-code problem stays beside it. |
| B. Short coded fields: reaction outcome, patient sex, drug role, action taken, reporter qualification. | Fixes the general cause for the fields people read. One schema change. |
| C. Every coded field, including route. | Largest table, little extra value. |

Recommendation: B.

**Could it split?** Not required, because the items are independent and small. If a lane stalls, split FAERS and GWAS (schema changes) from EMA and trial text (no schema change).

**Size: M.** Each item is small, but it touches four adapter families, and the trial results path needs a design choice.

**Amendment text.**

```
## Amendment, 2026-11-06

Checked by reading data, not by running. EMA's `?` is in the source: a copy of the publisher's medicines file is valid UTF-8, has no replacement characters, and holds both `?10 g/dl` in 69 indications and real `≥` elsewhere. Leave the value, as the Proof says. A `?` cannot be spotted without guessing from text, so nothing is marked in replies. Add one line to `docs/cautions.md`.

Trial escapes such as `\<` are in the source's own values.

"One shared text cleaner" means one module with two named functions, entity decoding and backslash unescaping, called only for EMA and ClinicalTrials.gov text. It is not applied in `NativeText` or to every source, because backslashes and entities are not markup everywhere. It stays in `biomcp-adapters`, with no change to BioData.

All cleaning happens when a reply is read. Stored rows, word indexes and the bundle format do not change, so existing snapshots stay valid.

Trial result text deserializes straight into public types with no mapping step. Clean it in `ctgov/detail/results.rs`.

The Proof checks JSON for clean text. Markdown keeps the escaping that 0348 requires.

FAERS: reaction outcome, patient sex, drug role, action taken and reporter qualification get words beside the code, copied from openFDA's published field reference.

GWAS: `p_value` is left out only when it is zero and `exact_p_value` is present.

Land after 0718, which edits nearby EMA and FAERS files.
```

## 0676 — Hosted schemas hide fields whose every source is off

**Current state.** Not started. No lane, no branch. `pm next` shows it as ready, with a claim that collides with every active lane.

**Detail (read, not run).**

- **The shared projection already exists, so this is smaller than the ticket says.** `catalog.rs::argument_allowed` reads an `x-required-component` fact on a field. `caller_schema` removes such fields from tools. `hosted.rs:94,181,192` use it for help, and `next_allowed` (`:318`) uses `request_allowed` for `next`. So annotating a field updates schema, help and `next` together, with no new projection.
- Only 5 fields and 1 map carry the fact today, all in `search-adverse-event` (`meddra`, `gmdn`).
- **One gap in the mechanism:** the fact names one component. A field answered by two sources needs "hide only when all are off". That is a small change to `argument_allowed` and `required_components`, not a new system.
- **Real size (rough, from a script over `request.json`).** 20 commands have two or more sources, with about 102 non-shared fields, and about 75 of those mention a source in their text. That is an upper bound, not the work.
- **The ticket's inventory looks wider than hosting's real need.** It lists ChEMBL selectors, recall fields, OLS and SEER, yet those sources are not noncommercial or licensed in their `Terms`, so `restricted` should leave them on. By label, `restricted` turns off WHO PQ, WHO IVD, DDInter, ClinPGx, Semantic Scholar, ORCID, OncoKB, AlphaGenome, OMIM and some components (CADD, dbNSFP, COSMIC, SNPedia, MedDRA, GMDN). The m27096 mail was not read, and hosting may disable more by configuration.
- **No rights exposure.** A disabled source is not called. This only changes what is offered.

**Does it need to precede the hosting commit? No.** It changes what is offered, not what is served, and `Keeps` leaves refusals unchanged. Its cost is that hosted testers may hit fields that are refused or come back empty, and could report them as defects. Tell the evaluators to expect that. It stays before 0640 and 0293, as 0293's Depends line says. 0654's amendment still says "after the cut", which is out of date against 0293's amendment.

**Risks and the smallest derisk.**

| Risk | Smallest derisk |
|---|---|
| Building a new "field to sources" framework when the fact and the reader already exist. | Reuse `x-required-component`, extended to a list. |
| Scope: "every owner", most of it for sources hosting serves. | Do hosting's real off set first, in the owners that matter: drug (WHO, DDInter), diagnostic (WHO IVD), pgx, author, article. Ask hosting for the sources it disables beyond `restricted`. |
| **Collision is the main cost.** Annotations sit on query types in `biomcp-types/src`, and they regenerate schemas that every lane regenerates. | Start after the bench scored build and after 0723. Split by owner, as `Defers` allows. |
| The ticket's claim is `crates/biomcp-query/src/`, but the facts live in `crates/biomcp-types/src/`. `pm next` therefore reports the wrong collisions. | Correct `Changes`. |
| 0723 changes which sources take `product_code`, and 0726 adds cell-line filters. Annotating before they land means doing it twice. | Order after 0723. 0726 annotates its own fields. |
| **0727.** Its per-request exclude list changes `excluded`, which this projection reads. Both touch `catalog.rs` and hosted filtering, in different functions. If 0727 feeds `catalog::excluded`, then tools, help and `next` follow it per request. If it filters elsewhere, a caller's list and schema disagree. | Have 0727 feed `catalog::excluded`. One sentence in its ticket and one case. |
| Hiding a field changes the tool list, which is cached by clients and counted by the 0683 budget. It can only shrink. | No action. |
| The Proof says "for each owner", which is broad and easy to spiral on. | One case per owner at `restricted`, plus 0723's moved filter-by-source walker pass, which replaces per-field cases. |

**Should it split? Yes, by owner, when a lane starts it.** The first slice extends the fact to a list and does drug and diagnostic, proving the pattern. The rest follow in parallel without touching shared code.

**Size: M** for hosting's real off set. **L** if all nine owners and all fields stand.

**Decision needed (PM).** Scope: every field by label, or hosting's real off set first. Recommendation: the real off set first, then the remaining owners only if the walker pass finds them. Confirm the off set with hosting.

**Amendment text.**

```
## Amendment, 2026-11-06

Read in code, not run. The shared projection already exists: `argument_allowed` reads `x-required-component`, and it already drives the tool list, help and `next`. This ticket adds facts to it and builds no new projection. Today the fact names one component on 5 fields. Extend it to a list, hiding a field only when every listed source is off.

Facts go on the query types in `crates/biomcp-types/src/`, so Changes covers that crate.

By their terms labels, ChEMBL, FDA recalls, OLS and SEER stay on at `restricted`, so start from the sources `restricted` and the deployment really turn off, confirmed with hosting. Other owners follow only if the check below finds them.

This ticket owns the check moved from 0723: with these facts, the enum walker sends each field with each source that declares it and fails if the command refuses it. One case per owner at `restricted`, not one per field.

The ticket changes what is offered and never what is served, so it does not gate the hosting deployment. It lands before 0640. Order it after 0723. Split by owner when a lane starts it: the first slice extends the fact and covers drug and diagnostic.
```

# Cleanup

## 0711 — Code docs build without broken links

**State.** Not started. The problem is on main.

**Detail.**
- j32021's log (RUN, at f33f42a55) shows rustdoc could not finish biomcp-types and biomcp-adapters.
- It lists 18 "this URL is not a hyperlink" errors, unclosed HTML tags (ACCESSION, CAid, CODE, CURIE x2, disease, drug, exact, file x2, FILE) and unresolved links `loci` (`error.rs:83:71`) and `sections` (`error.rs:175:49`). That is about 30 known errors. Later crates are unknown because the run stops.
- Root cause of many: `singleton!` at `crates/biomcp-types/src/response/error.rs:13-15` writes `#[doc = $literal] #[serde(rename = $literal)]`, so one string is rustdoc text, wire value and schema enum.
- **Answer: fix the macro, never the strings.**

**Risks and derisk.**
- Doc comments feed schema descriptions: run `current-schema-check` and `schema-release-check`.
- More errors may hide in later crates: fix the first two, then one `cargo doc` job shows the total.
- The gate step collides with 0694 in the Makefile: land after or with it.
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` cost is unmeasured: read it from the first job.

**Size.** S-M. "Under an hour" is optimistic.

```
## Amendment, 2026-11-06

Many of the reported errors come from one place. The `singleton!` macro at `crates/biomcp-types/src/response/error.rs:13` uses each usage string as its rustdoc text as well as its wire value and schema enum. Fix the macro with a fixed doc line or a code span. Do not change the strings, which are wire values and asserted output.

Doc comments on schema types also feed schema descriptions, so `current-schema-check` and `schema-release-check` must pass after the fix. The first run stopped after two crates, so later crates may hold more errors than the roughly 30 reported.
```

## 0712 — Release binaries carry no build paths

**State.** Not started. W42 recorded 510 matches on the landing host's binary.

**Detail.**
- `profile.trim-paths` is refused by cargo 1.95.0 (RUN), so the route is `--remap-path-prefix`.
- `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS` would replace the Windows `+crt-static` rustflags in `.cargo/config.toml`, with no error (from memory).
- `[build] rustflags` is ignored when target rustflags exist (from memory).
- Safest place: `release/vendor.py environment()`, which writes a temp `config.toml` and knows the vendor path. Add `[target.'cfg(all())'] rustflags`, which cargo joins with the Windows entry (from memory).
- Remap the vendor stage, `CARGO_HOME` and the source root.
- C code from `cc` (rusqlite bundled, ring) is not remapped. Add a prefix-map only if the proof finds paths.

**Risks and derisk.**
- Windows loses crt-static: a tiny temp crate with both configs and `cargo build -v --target x86_64-pc-windows-msvc`, reading the printed rustc line. No Windows host needed.
- False matches in containers (root `/source`, `HOME` `/tmp/release-home`): search `<root>/crates`, the vendor stage and `CARGO_HOME`.
- Plain `cargo build --release` misses the vendored path: prove through `release/artifacts.py build`.
- `cargo()` is shared with `scripts/trials.py`: limit the flags to the release profile.

**Size.** S-M.

```
## Amendment, 2026-11-06

`profile.trim-paths` is refused by cargo 1.95.0. Use `--remap-path-prefix` in the temporary `config.toml` that `release/vendor.py` writes, as `[target.'cfg(all())'] rustflags`, where the vendor path is known.

Do not use `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS`. Either replaces the target `rustflags` in `.cargo/config.toml`, and the Windows build would lose its static C runtime with no error.

The check belongs in `release/artifacts.py build()` so every platform runs it. It searches for the source `crates` path, the vendor folder and `CARGO_HOME`. In containers the root is `/source`, so a search for the bare root gives false matches. The Proof runs through `release/artifacts.py build`, not plain `cargo build --release`. The section preflight in 0640 is the Windows and Intel proof.
```

## 0713 — Container builds survive removed package versions

**State.** Not started. Nothing is broken today.

**Detail.**
- RUN: bookworm is oldstable 12.15, released 11 Jul 2026, with no Valid-Until in the header.
- RUN: all four pins (`ca-certificates=20230311+deb12u1`, `openssl=3.0.20-1~deb12u2`, `libssl3=3.0.20-1~deb12u2`, `debconf=1.5.82`) are in bookworm main.
- `release/toolchain.json` has **no snapshot date key**, though the ticket Proof reads as if it does.
- RUN: `http://snapshot.debian.org/archive/debian/20261101T000000Z/dists/bookworm/Release` redirects (302, relative) to the same 12.15 content with no https upgrade. An arm64 libssl3 pool file also resolves.
- The slim image has no CA certificates, so sources must be plain http, with trust from the signed Release.

**Answer on the pins.**
- Add a `debian-snapshot` key.
- Add a third build arg at `release/linux.sh:88-89` and `release/matrix.sh:63-64`.
- Extend `release/toolchain.py check()` and the tests at `release/test_linux.py:199-200` and `release/test_toolchain.py:56-58`.
- Digest checks are unaffected.

**Risks and derisk.**
- The Proof can't fail as written: sharpen as below.
- The gate has no real docker build (fake docker in `test_linux`): one real build job.
- The snapshot service can be down: the rebuild fails closed. Accept that, with no fallback.

**Size.** S-M.

```
## Amendment, 2026-11-06

All four pins are in the normal Debian archive today, so a build against it proves nothing about removed versions. The Proof changes: the build's package sources point only at the dated snapshot, and the build log shows every package fetched from it.

`release/toolchain.json` holds no snapshot date yet. Add a `debian-snapshot` key next to the pins, pass it as a build argument from `release/linux.sh` and `release/matrix.sh`, and extend `release/toolchain.py` and its tests.

Use plain `http://snapshot.debian.org/archive/debian/<date>/` for bookworm main only, with `Acquire::Check-Valid-Until=false`. The slim image has no CA certificates before this step, and the signed Release file provides trust. There is no fallback to the live archive: if the snapshot is unavailable, the build fails.
```

## 0714 — Release profile is set and the hot path measured

**State.** Not started. `Cargo.toml` has no `[profile.release]`.

**Detail.**
- The routine gate and coverage never use the release profile.
- `release/gate.py:137,139` also builds case-runner and tool-runner in release, so link-time optimization is paid three times per platform. That sits on top of the cold matrix (99 minutes in j29497) and Intel at 119m44s of 120.
- `inspect()` requires `opt_level == "3"`, so `s` or `z` fails.
- `catch_unwind` in `mcp/worker.rs`, `biomcp-fetch/src/batch.rs` and `biomcp-fetch/src/cache/contention.rs` rules out `panic = "abort"`.
- A16 evidence: `command/transport.rs with_cancellation`, `store/mod.rs`, `ema/mod.rs:57`, `who_ivd/search.rs:17`, `who_pq/mod.rs:50`.

**Risks and derisk.**
- LTO pushes Intel over its limit: land after 0697 and record build time. If it is bad, keep the default profile.
- Connection setup can't be measured on recorded replies: local TLS stub, or say unmeasured.
- Timing is noisy on a shared host: book cores.
- Spiral risk: one measurement, no kept harness.

**Size.** S for the profile, M with measurement.

```
## Amendment, 2026-11-06

Keep `opt-level = 3` and unwinding. `release/artifacts.py inspect()` refuses any other opt level, and `catch_unwind` in `crates/biomcp/src/mcp/worker.rs`, `crates/biomcp-fetch/src/batch.rs` and `crates/biomcp-fetch/src/cache/contention.rs` needs unwinding.

Each release section builds three binaries in this profile, so link-time optimization is paid three times per platform, and the Intel build has already used 119m44s of its 120 minute limit. Land this after 0697 raises the limit, and record the build time next to the size.

Recorded replies open no connection, so they cannot measure connection setup. Measure it against a local TLS stub, or state it as unmeasured. This is one measurement: no harness stays behind.
```

## 0715 — Sweep per-ticket proof sediment from tests

**State.** Not started. Mis-scoped.

**Detail.** The source finding (record lines 791-795, finding 19) says "83 of 125 Make goals... named for single tickets. 90 of 183 scripts are referenced by nothing... Smallest fix: after 1.0, sweep scripts with no caller", and "Test sediment sampled lightly".

**The pile on main (heuristic).**
- 187 scripts: 51 `check_*.sh`, 10 `*_job.sh`, 79 with no caller by name, 16 called only from the Makefile.
- 172 Make goals, 73 with no reference beyond their definition.
- 395 yaml in `tests/cases`, 18 tool-cases, 7 retained-cases.
- 2,481 files under `tests` and `crates/biomcp/tests`; 46,928 Rust lines in `crates/biomcp/tests`; 25 ratchet files (4,138 lines); 572 `#[test]`.

The 79 is **not a delete list**. It includes `check_cache_process_load.sh`, which 0641 names.

**Why the Proof can't fail.**
- Floor 80.00, less 0.2 in `scripts/coverage.sh:7-14`, is 79.8 against 84.54 measured.
- `release/mutations.py` returns `int(code not in (None, 0, 2, 3))`, with the comment "Survivors and native timeouts are evidence."
- A test-only sweep leaves only `RULE_FILES` in scope.

**Risks and derisk.**
- Collides with every lane adding tests: run last, after bench and hosting commits.
- A lost test goes unnoticed: list each removed test with its covering case.
- No bound: split and time-box.

**Size.** L as written, M rescoped.

```
## Amendment, 2026-11-06

This ticket's evidence is Make goals and scripts. The source finding counted goals named for single tickets and scripts nothing calls, and it sampled tests only lightly. The scope becomes, in order:

1. Scripts and Make goals with no caller. A name search gives a starting list, not a delete list: a script named by an open ticket stays until that ticket closes.
2. Tests, one folder at a time and time-boxed. Stop at the first folder that gives nothing.
3. Machine names and workspace paths in tracked files outside `sdlc/`, which 0279 cleared and which have come back. Add one search for them to `scripts/check_docs.py`.

This runs last among cleanup, after the bench and hosting commits, because every lane adds tests.

The Proof changes, because the current one cannot fail. The coverage floor sits about 4.7 points below measured coverage. The mutation run counts survivors and timeouts as passing, and a test-only change leaves only the rule files in its scope. New Proof: coverage after the sweep is no lower than on the commit before it, less 0.2 points; each removed test is listed with the case that covers the same behavior; and the routine gate passes. The mutation clause is dropped.
```

## 0716 — Wheel ships one copy of the binary

**State.** Not started. The problem is on main.

**Detail.**
- `release/artifacts.py wheel()` lines 90-91 write two deflated copies: `biomcp` and one named for the distribution. `inspect()` near line 133 requires both with the same hash.
- The second serves `server.json` (identifier `biomcp-cli`, runtimeHint `uvx`, `serve`).
- `release/gate.py:185-193` proves it with `uv pip install`, never real uvx. `release/published.py:68-72` requires that exact shape.
- RUN: the 0.9.3 wheel holds only `biomcp` (40.4 MB raw, 17.5 MB compressed), so `uvx biomcp-cli` never worked publicly. 0.9.3 wheels are 16-18 MB.
- RUN: the MCP Registry has no `io.github.genomoncology/biomcp` (404), so no user depends on a `biomcp-cli` command.
- Registry schema 2025-12-11 has no entry-point field.
- VS Code's `mcpManagementService.ts:121-175` (read) joins runtimeArguments, `identifier@version`, then packageArguments. So a positional `--from` plus packageArguments `biomcp`, `serve` gives `uvx --from biomcp-cli@1.0.0 biomcp serve`. uv 0.13.5 parses that (RUN). A `named` `--from` would be wrong.
- The schema does not define join order, and other clients are unknown.

| Option | What | Cost | Risk |
| --- | --- | --- | --- |
| A | `--from` in `server.json`, one copy | Change sites below | Each client's join order; only VS Code read |
| B | Python console-script launcher | New code | Windows exec, stdio, Ctrl-C |
| C | Keep two copies, close the ticket | About 16 MB per wheel (estimate; candidate not measured) | None |
| D | sh wrapper | Small | No Windows |
| E | Links | None | Impossible in a zip |

Every option changes the Outcome as written. A and D stop `uvx biomcp-cli` working as a bare command (A makes it `uvx --from biomcp-cli biomcp`), and C keeps two copies. Only B meets it literally, and B is new code on the launch path. So the PM has to choose.

**Recommendation: C for 1.0.** The wheel stays under PyPI's default 100 MB file limit (from memory). A cleanup should not add a change that only hosted Windows and Intel can prove and that could break the first registry entry. If the PM wants the smaller wheel now, take A.

**Change sites for A.** `release/artifacts.py:90-91,133`, `release/gate.py:186-193`, `release/published.py:68-72`, `release/test_gate.py:234`, `release/test_licenses.py:52-53`, `server.json`.

**Derisk for A.** One offline `uvx --from` run on the local wheel, plus trying `mcp-publisher validate` as a read-only check (unverified). Run 0697's twine after the shape is final.

**Size.** C is S, A is M.

Recommended text (C):

```
## Amendment, 2026-11-06

Found while sizing this ticket. The published 0.9.3 wheels hold only `biomcp`, and the public MCP Registry holds no BioMCP entry yet, so the second copy serves only the new `uvx biomcp-cli serve` route in `server.json`. The registry schema has no entry-point field. The one-copy route is `uvx --from biomcp-cli@VERSION biomcp serve`, written with a positional `--from` in `runtimeArguments`. The schema does not define the order in which clients join those parts, and it can only be proved on hosted Windows and Intel runs.

Decision: the wheel keeps both copies for 1.0, at a cost of about 16 MB per wheel. Close this ticket without a change, and reopen the one-copy route after 1.0 if the join order holds for a second client.
```

Alternative text (A):

```
## Amendment, 2026-11-06

Found while sizing this ticket. The published 0.9.3 wheels hold only `biomcp`, and the public MCP Registry holds no BioMCP entry yet, so nothing depends on a `biomcp-cli` command. The registry schema has no entry-point field.

Decision: drop the second copy. `server.json` uses `runtimeArguments` with a positional `--from` and `packageArguments` of `biomcp` and `serve`, which gives `uvx --from biomcp-cli@VERSION biomcp serve`. Change `release/artifacts.py`, `release/gate.py`, `release/published.py` and their tests. The Proof adds one offline `uvx --from` run on a local wheel, because the existing check uses `uv pip install` and never runs uvx. The section preflight in 0640 is the Windows and Intel proof.
```

# Tooling and tests

## 0694 — Tool proofs run in the routine gate

**State.** Ready, not started. Collides with nearly every lane through the Makefile.

**Detail.**
- `scripts/landing_gate.py` STAGES has no proofs stage.
- `crates/biomcp/Cargo.toml` sets `[[test]] name="tools" test=false harness=false`, so the proofs run only from `release/smoke.py:147-152`. That script sets `BIOMCP_TEST_EXECUTABLE`, `BIOMCP_TEST_FIXTURES` and `BIOMCP_TEST_PYTHON` and uses 2 threads.
- j32021 (RUN): `PROOFS exit=101 secs=72`, 34 passed, 3 failed, 1 ignored, in 56.55 s.
- Failed: `mcp-http/readiness-rejects-eof-and-hung-child`, `live-source-checks/retained-drift-witnesses`, `schema-discovery/rejects-bad-cases`. This ran before 0686. Whether the first two are environment or a second cause is unknown.
- The proofs take about a minute. The rest of the 4.5 minutes is compiling the tools target.

**Python, three truths.** 3.12 in `release/toolchain.json` and the hosted exact check. 3.13 in `sdlc/pm.json` `ticketsGate` and 36 files, including `Makefile:391,581,588,592` (line 391 holds an absolute user path) and `scripts/test_gate_stages.py`. 3.9.6 as the default on the landing host.

**Decision: set the toolchain to 3.13 (recommended).** The routine gate and lanes stay put and only the hosted side moves. cp313 is already in manylinux use.

**Risks and derisk.**
- Red on day one: first step is one job of `cargo test --locked -p biomcp --test tools -- --proofs` with the `smoke.py` environment.
- Line numbers moved: `scripts/check_front_map.py` bare asserts are at 73, 74, 78 and 87.
- Makefile collisions: land in a batch after bench, with lane goals dropped.

**Size.** M.

```
## Amendment, 2026-11-06

First step: one job running `cargo test --locked -p biomcp --test tools -- --proofs` on main, with the environment `release/smoke.py` sets. The last run of this form was before 0686 landed: 3 of 38 failed (`mcp-http/readiness-rejects-eof-and-hung-child`, `live-source-checks/retained-drift-witnesses`, `schema-discovery/rejects-bad-cases`). Each remaining failure is fixed or ticketed before the stage joins the gate.

`release/toolchain.json` says Python 3.12, while `sdlc/pm.json` and 36 files say 3.13. Set the toolchain to 3.13, so the routine gate and open lanes stay unchanged and only the hosted side moves. Do the file sweep after the bench build.

The bare asserts in `scripts/check_front_map.py` are now at lines 73, 74, 78 and 87.
```

## 0695 — Noncommercial captures become synthetic

**State.** Not started.

**Detail.**
- WHO: `tests/replies/drug/who-dates/who-pq/cb08866b...-0.bin` is a 129,551 byte real CSV. `who-synthetic` (562 bytes) is already synthetic.
- **New: the case holds real data too.** `tests/cases/drug/who-dates.yaml` asserts 657 rows, a total of 51 for 2020, and has 58 lines matching real product references (RUN). The capture and the case must change together. `scripts/check_who_dates.sh:78` names this case. `who-sync.yaml` has 2 matches I did not inspect.
- Semantic Scholar: 39 files in 17 folders, about 21 real by folder name. Files holding only 429 errors have no rows and stay.
- Replies are keyed by request hash, so body edits keep the key (RUN, one header).
- Not in the ticket: 7 cBioPortal, 1 VAERS. I did not judge their rights.

**Lane overlap (RUN).** Only 0717 touches Semantic Scholar replies (3 files). 0693 (21 replies) and 0722 (3) do not by path. 0689 touches no replies but edits `article_rights/semantic.rs`. None touch WHO replies.

**Risks and derisk.**
- The grep can't fail without a list: collect paper IDs, DOIs and product references first.
- Synthetic rows change asserts: keep row shapes and rewrite the case in the same commit.
- The case has `live_check: public`: confirm the live check still means something with synthetic rows (not read in detail).

**Size.** S-M. WHO can start now; Semantic Scholar waits for 0717.

```
## Amendment, 2026-11-06

Before editing, list the real rows to remove: paper IDs and DOIs from the Semantic Scholar replies, and product references from the WHO capture. The Proof greps for that list, because a search for "no real row" with no list cannot fail.

`tests/cases/drug/who-dates.yaml` asserts 657 rows and holds real product references, so the case is rewritten in the same commit as its capture, and `scripts/check_who_dates.sh` is rechecked. Replies are keyed by request hash, so editing a body keeps the key. Semantic Scholar replies that hold only 429 error bodies have no rows and stay. The Semantic Scholar part follows 0717, which changes three of those replies.
```

## 0696 — Plugin skill ships expanded

**State.** Not started. The problem is on main.

**Detail.**
- `skills/SKILL.md:6` holds `{{front-map}}`.
- `crates/biomcp/build.rs:328-334` requires exactly one marker and replaces it.
- `release/workflow.py plugin()` (about 105-130) archives raw `skills/`.
- `release/published.py plugin()` (140-165) never reads the skill.

**What the ticket misses.** `.claude-plugin/marketplace.json` has `source: "./"`, and 0294 C4 says plugin users are served from the repo root, so the raw file reaches them whatever the zip holds. `claude plugin details` could prove the skill loads with no model; not run.

**Options.**
- (a) Commit generated `SKILL.md`. Every help change then collides.
- (b) **Recommended.** Marker in an HTML comment plus a plain pointer line in the repo file, expanded in zip and binary.
- (c) Zip only. Marketplace users stay on the raw marker.

The PM can take (b): it is inside the approved outcome.

**Risks and derisk.**
- The Proof ties to 0640: split it.
- `build.rs` must still see one marker: match the whole comment line.

**Size.** S-M.

```
## Amendment, 2026-11-06

The plugin reaches users two ways: the release zip and the repository root, because `.claude-plugin/marketplace.json` uses `source: "./"`. Fixing only the zip leaves `{{front-map}}` raw for marketplace users.

Keep the marker in an HTML comment in `skills/SKILL.md`, with one plain line beside it pointing to `biomcp skill install`. `crates/biomcp/build.rs` and `release/workflow.py plugin()` expand it for the binary and the zip.

The Proof splits. A unit assertion in `release.test_workflow` shows the zip's skill holds no marker, and one native build with an unpacked plugin shows the skill loads. The five-platform staged check belongs to the plugin section in 0640.
```

## 0697 — Public release text and limits

**State.** Not started. Five separate items.

**Detail.**
1. **Limits.** `release.yml:96` and `recover-section.yml:43` set `timeout-minutes: 120` on a job shared by all five platforms.
2. **Wording.** `release/workflow.py release_notes()` (about 130-145), line 187 and `release/notes.py:22`.
3. **macOS floor.** 11.0 is absent from the notes.
4. **twine.** No pin, not used anywhere. METADATA 2.1 is hand-written.
5. **AlphaGenome notice.** `release/notices.py`, with `crates/biomcp-fetch/proto/LICENSE` and `crates/biomcp-adapters/src/alphagenome/generated/*`.

**Installer byte check.** The ticket doesn't define it. The review (W29) says `release/published.py`'s installer check "does not compare with the staged bytes". So it is that comparison. A check at `release/workflow.py:182` would compare its own `copy2` and can't fail.

**Risks and derisk.**
- Workflow edits are provable only by actionlint until hosted: make the preflight the proof.
- twine depends on wheel shape: run after the 0716 ruling.
- Five items can stall: limits go first, because 0714 needs them.

**Size.** M, splittable.

```
## Amendment, 2026-11-06

The `recover-section.yml` limit is now at line 43. Both limits sit on a job shared by all five platforms, so one value applies to all.

The installer byte check is a comparison in `release/published.py` between the fetched installer and the staged `install.sh`, which that check does not do today. Before the tag it runs against a locally served copy.

The limits land first, because 0714 needs them. `twine check` runs after 0716 settles the wheel's shape.
```

## 0729 — Focused test runs refuse zero selection

**State.** Branch at 55e071b77, 2 commits, 15 behind. Its only job, j32559, was cancelled, and it is not in pm Running. Stalled but nearly done.

**Detail.** The diff (RUN) touches the six paths in the amendment. `is_filtered_out` is public in libtest-mimic 0.8.2.

**Probable deeper cause of "five dependency-heavy builds, 26m55" (inference, not measured).** `trials.py` without `--runner` gets a fresh random vendored path per process, and cargo fingerprints include absolute paths of non-path sources. Using the vendor environment only for release would remove it.

**Risks and derisk.**
- The inference may be wrong: one job, two `trials.py` runs in a row, compare times.
- The branch is behind: merge main and run the gate.

**Size.** S.

```
## Amendment, 2026-11-06

The ticket's `tools/` is not a folder. The change sits in `crates/biomcp/tests/tools.rs`, `crates/biomcp/tests/runner/case_main.rs`, `crates/biomcp/tests/runner/proof.rs`, `scripts/trials.py` and `scripts/test_trials.py`, plus `docs/development.md`.
```

## 0649 — Pi namespace discovery

**State.** No change started.

**Detail.**
- The 2,000 character limit was a slice the model chose, not a protocol limit.
- The corrected harness is 9266ff73c on `ticket/0632-pi-codemode`, and j30474 passed. **It is not on main (RUN)**: `scripts/typed_harness_smoke.py` differs by 16 added lines.
- Server instructions are at `crates/biomcp/src/mcp/protocol.rs:7-22`.
- Likely outcome: no server change.

**Risks and derisk.**
- Main's harness stays wrong: bring the commit over.
- The harness runs the server over ssh with a Pi client: reuse the j30474 setup.

**Size.** S.

```
## Amendment, 2026-11-06

The 2026-11-06 amendment on 0293 puts this ticket before the final rehearsal, so the line saying it does not gate 1.0 no longer applies.

The 2,000 character limit was a slice the model chose, not a protocol limit, so no server change is expected. The corrected harness is commit 9266ff73c on `ticket/0632-pi-codemode` and is not on main. Land it, run the smoke once, and close with the result.
```

## 0641 — Concurrency tests under repeated load

**State.** The lane's 4 commits moved to 0728 and landed. The lane is 20 behind with all jobs cancelled, including j32394 and j32395, which 0728 cites as proof. pm shows them cancelled after about 36 minutes, and their logs are empty, so **the PM should confirm that citation**. I rechecked: `pm job list 0641 --all` shows both cancelled, exit 130, and 0728's Proof line names both as its proof. 0728 is complete on main.

**Detail.** Not started: one Make goal, the named test list, the W24 measurement, the nightly entry (pm repo owner's). Sources:
- `scripts/cache_process_load.py`
- `check_cache_process_load.sh`
- `check_fixed_cache_first_open.sh`
- `cache_installed_first_open.py`
- `gencc_publisher_load.py` (10 rounds, 1.5 x `PM_JOB_CORES`)
- `deadline_setup_load.py` (10 repeats, 1.5x booked cores, hard-coded ssh)
- `sdlc/records/2026-11-05-timing-tests-under-load.md`

**Risks and derisk.**
- Load set from machine cores hurts other jobs: 1.5x booked cores.
- The wheel proof needs 015aea0a7's Linux AMD wheel with no path in the repo. j30327 was 1 of 24, so failure is likely, not certain: name the path or rebuild.
- It can hold the whole release: split and start now.

**Size.** M-L.

```
## Amendment, 2026-11-06

The branch's four commits moved to 0728 and landed. Nothing else is on it, so release the lane and restart from main.

The work splits into three parts, in order: one Make goal with the named test list, the W24 measurement, and the nightly entry. Load is 1.5 times the job's booked cores, not the machine's cores, because builders are shared. The wheel proof names where its wheel comes from before it runs.
```

## 0670 — Moderate risk timing tests

**State.** Not started.

**Detail.** The fixes need only the table. Only the Proof needs 0641.

**Risks and derisk.**
- 0690 edited `http_limits.rs` and has landed, so recheck that row's line number.
- `scripts/deadline_setup_load.py` names a fixed host: don't reuse it as is.

**Size.** M.

```
## Amendment, 2026-11-06

The fixes need only the table, so they can start before 0641. Only the Proof waits for 0641's goal. Line numbers have moved: `retries.rs:145`, `freshness.rs:7,98`, `transport.rs:365,395`, `grpc_worker.rs:317,427,508`, `download.rs:168` and `hgnc_sync.rs:29`. `scripts/deadline_setup_load.py` is not reused as it stands, because it names a fixed remote host.
```

# Release

## 0640 — Release sections pass on main before the cut

**State.** `ticket/0640-main-section-preflight` at 69c443ec6, 2 commits, doc only, 94+ behind, j30684 passed. Main's `release/qualification.md` is still old (names ed9c36a).

**Stale on the branch.**
- It says the recovery action must be parameterized. 0666 did that (`main_revision`, `main_tree`).
- Its c9bd901 lines are superseded.
- It needs rows for 0696, 0697's twine, 0712, 0713 and 0716.
- It names machines at lines 36 and 46 (RUN), which breaks 0279 and the rule that jobs name no machine unless they must.

**Dependence.** 0696's staged proof depends on it directly, and 0697's twine needs a named section. The branch doc has a staged scan row (line 97), but the ticket's list omits stage, staged scan, attestation and staged Homebrew (review finding 1).

**Risks and derisk.** The doc goes stale again: land it now, and add rows as tooling tickets land. Only the preflight is held.

**Size.** Doc S, preflight L.

```
## Amendment, 2026-11-06

The document on this ticket's branch is stale. 0666 already made the recovery action take a main revision, and its candidate lines are superseded. Refresh it, take machine names out of it, and land the document now: only the preflight runs are held.

Stage, the staged artifact scan, attestation and staged Homebrew are sections too, and the document lists them. Rows change as 0696, 0697, 0712, 0713 and 0716 land.

Tooling tickets land on proof from a local real host. The preflight is their Windows and Intel proof, and a failure there reopens the ticket that caused it.
```

## 0293 — Final release rehearsal

**State.** Held. No work until 0640.

**Detail.** Depends has 78 entries (W44 flagged 44), including completed 0728. pm daily's Blocked line lists 33 open tickets. My comparison of the two lists was buggy, so that match is not verified.

**Risks and derisk.**
- Depends is too long: `pm lint` already reports it, so trim when W44's edit is applied.
- The pm status line misleads on the 12 findings: update it at the next progress write.

**Size.** L, external. **Amendment:** none needed.

## 0294 — History replacement

**State.** Held. Rehearsal j30538 passed at superseded c9bd901 and must rerun on the final candidate.

**Detail.** The amendment (tag first, fast-forward after publication) overrides older order text. W8's read-only check is pending.

**Risks and derisk.**
- Internal text goes public: the search below.
- Noncommercial captures go public: 0695 lands first.

**Size.** M, Ian-gated.

Append to the existing amendment:

```
Before the append, a search of tracked files outside `sdlc/` finds no machine name or workspace path, as 0279 required, and 0695 has landed. The rehearsal reruns on the final candidate, because the last pass was on a superseded one.
```

## 0401 — Land docs site after release

**State.** `origin/docs/site` at edbd9f7ae (2026-10-28), 41 ahead, 704 behind, 1,282 files, 58,340 insertions, README +79/-28.

**Detail.** Per W33 the refresh does not wait for 0294. Examples go stale until bench surfaces settle, so refresh after the hosting commit. I found no Node pin in `release/tools.py` or `release/toolchain.json`, although the ticket cites the tool manifest.

**Risks and derisk.**
- 704 commits of drift: refresh once.
- No Node pin: add it at refresh.

**Size.** M-L. The docs lead owns the branch.

Append to the existing amendment:

```
The refresh follows the hosting commit, because examples go stale until the bench surfaces settle. The tool manifest holds no Node pin, so add one with the refresh.
```
