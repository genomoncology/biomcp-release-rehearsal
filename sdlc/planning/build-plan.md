# BioMCP 1.0 build plan, 2026-11-06

**Rules changed the same day.** `skills/build-efficiency.md` wins over anything below. Ian ruled that the full suite runs only at final release. So: developers run targeted checks; a pre-review routine run only for risky tickets (0689, 0691, 0717, 0727); each batch lands through the short landing check (0731); at most two runs of the longest check per ticket. Jobs name no machine; the Linux builder is closed to BioMCP. The bench takes the macOS build. For Linux x86-64, hand-start the release workflow on GitHub. 0731 lands first; 0732 replaces the nightly live check.

For the build coordinator. Read against main at 502de0e2c and the lane heads of that afternoon; main then moved to e196c73fe, where 0690 and 0692 landed. Per-ticket detail is in `records/2026-11-06-ticket-review.md`, and the amendments are on the tickets. Order and rulings already settled by Ian are not reopened here.

Main already holds 0682, 0683 with the tool list, 0690 and 0692, so none of them needs work.

## Rules for every lane

- **Stop proof tooling now.** Per-ticket scripts and Makefile goals do not land, and no new ones get written. Case and test files stay. Run the same commands with `pm job run KEY SHA -- CMD`. These are the scripts and goals to drop, by lane:

| Lane | Scripts | Makefile goals |
|---|---|---|
| 0689 | `scripts/check_article_rights.sh` | `article-rights-*` |
| 0691 | `scripts/check_article_pool_mapping.py` | `article-pools-*` |
| 0693 | | `typed-followups-*`, `typed-owned-followups-*`, `status-totals-*` |
| 0710 | | `study-flags-*` |
| 0717 | `scripts/check_search_drug_names.sh`, `scripts/search_drug_names_baseline.py` | |
| 0718 | `scripts/check_source_age.sh` | `source-age-*` |
| 0719 | | `trial-interventions-*` |
| 0720 | `scripts/seer_live_probe.py` | `seer-*` |
| 0721 | | `recurrence-forms-*`, `recurrence-indel-check` |
| 0722 | `scripts/check_author_artifact.py` | `author-search-*`, `author-credential-review`, `author-artifact-review` |

  This also removes most of the merge conflicts between lanes, and keeps machine names and internal paths out of the public tree. 0690 and 0692 landed with their goals, so 0715 sweeps those.
- **At most two runs of the longest check per ticket.** A lane does not repeat a passed run because of a mechanical change. The reviewer's check is the review.
- **Generated files:** do not hand-merge schemas or `docs/reference/commands.md`. The lander runs `schema-generate` and `docs-generate` once per batch.
- **Fix general causes.** Nothing is tuned to the report's examples, and no new source is added.

## Machines

Jobs name no machine, with one exception: the macOS scored binary needs the macOS host. 0713's real container build needs a host with Docker, so let placement find one. The landing host has 8 slots and they are full, and the Linux x86-64 host has 2 and is reserved now. So bench lanes get slots first. Load jobs in 0641 book 1.5 times the job's cores, never the machine's, because builders are shared. Windows and Intel exist only in the hosted preflight, which waits for Ian.

## Wave 0: start now, all in parallel

| Ticket | Work | Size left |
|---|---|---|
| 0717 search drug names | Send back with the amendment. The lane passes on invented drugs but would not expand `sotorasib` on real data. | M |
| 0689 licensed article text | The five small fixes in the amendment, then one routine, then review. Stop widening the Semantic Scholar stored routes. | M |
| 0718 source data age | Read the Open Targets data version. j32887 passed. | S |
| 0721 variant recurrence | Raise the `limit` default and maximum, correct the page step source. | S |
| 0722 author search | Drop the four commits after 138c6ab18, which only add the artifact checker, and merge main. Cancel any job still running on it. | S |
| 0719, 0720 | Merge main, drop tooling, review, land. | S each |
| 0710 study flags | Merge main: it now conflicts in `crates/biomcp-fetch/src/lib.rs` and `docs/configuration.md`. Apply the amendment, then one routine. | S |
| 0693 status and totals | Rebuild from main with its five own commits. Split `try_from` for the length lint. | S |
| 0727 per-request sources | Start now, from main. 0690 landed, so that collision is gone. It is on the hosting path and nothing has started. | M |
| 0726 cell lines | Start now. Nothing else touches `cellosaurus/` except one 0718 file. | M |

## Wave 1: bench scored build

**Landing order.** Land what is ready in batches. Order is strict only where it says so.

1. 0710, 0718, 0719, 0720, 0721, 0722 in any order, as each is ready. Their conflicts are Makefile, generated files and the 0710 pair above, apart from one hunk in `drug/native.rs` between 0717 and 0718.
2. **Strict:** 0689, then 0717, then 0691. They all edit `article/mod.rs`, and 0689 and 0717 share `article/page.rs`. The 0691 lane does the last merge, computes `publish` after 0717's `reuse`, and keeps 0717's `term::search`.

Compile note for the lander: 0718 adds a field to `SourceNotice::RequestGuidance`. Git merges cleanly, then the build fails at new constructions in 0689 (2), 0710 (1) and 0719 (1). Add `data_date: None`.

**Gate for the scored build.** All of this, and nothing more:

1. 0689, 0691 and 0717 to 0722 are landed through `pm ticket land`, each batch passing the routine suite on its candidate.
2. `git diff 502de0e2c <commit> -- Cargo.toml crates/biomcp-fetch/src/cache.rs crates/biomcp-query/src/store/mod.rs` shows no change to `CACHE_NAMESPACE_VERSION`, `CACHE_NAMESPACES` or the BioData pin. No lane changes these today.
3. Build both binaries from that one commit the way 0730 did: `plain-cli-build`, Linux placed automatically, macOS on the macOS host. Retain them with checksums under the commit's folder.
4. One short live check on the Linux binary, run by hand as a job. It is a hand-off check, not a landing condition. A failure stops the hand-off and goes to the PM as a new bug:
   - `search trial` on a brand, generic and code name gives one total for `sotorasib`, `AMG 510`, `Lumakras`, and for two drugs not in the report (for example `imatinib`, `Gleevec`, `STI-571`; `pembrolizumab`, `Keytruda`, `MK-3475`).
   - `search trial "lung cancer"` still returns inside the time budget, cold.
   - `search author "KRAS G12C"` returns papers with no key.
   - `get drug` with Open Targets indications shows a data date and the FDA event comparison.
   - SEER incidence by the site name `pancreas`.
   - `variant recurrence` for a common KRAS variant on `msk_impact_2017` reports `all_pages_read: true`.
   - A trial with more than three interventions states how many more.

**After the gate:** the PM hands the binaries to the bench. 0693 follows the scored build, unless it lands first by chance. It merges clean in every simulated order.

## Wave 2: hosting fixes commit

This is the first main commit holding 0689, 0710 and 0727, on top of 0682, 0683, 0690 and 0692, which are in. If Wave 1 is done, it is main plus 0727. The freeze holds until Ian lifts it, and hosting is PM-only.

**0727 lands after** 0691, whose pool key keeps exclusion lists apart.

**Gate for the hosting fixes commit:**

1. Routine suite on the batch candidate.
2. `make hosted-surface-check` and `make hosted-http-load`, both existing goals, through `pm job` at that commit.
3. The same diff as bench gate step 2 shows no store, namespace or pin change.
4. The 0727 cases pass: a refused unknown name, a list that can only narrow, and two callers with different lists never sharing a stored pool.

**The PM tells the hosting PM before any deploy:**

- 0692 retires the `restricted` disable group. A config with `disable_sources: ["restricted"]` must move to `noncommercial` and `licensed`.
- The new header `x-biomcp-exclude-sources`, and that the front end strips client copies of it.
- `hosted_max_use` and the `x-biomcp-use` strip rules, now in `docs/configuration.md`.
- 0710 withholds 19 flagged studies at `restricted`. No refetch is needed.
- No store format, namespace or pin change.

Then Ian approves the deploy, and hosting validates. At validation, compare the hosted snapshot's study ids against the reviewed list once, read-only.

## Wave 3: after the scored build

Keep lanes full with this while the bench and hosting run. Most of it waits for Wave 1, because it shares files with the bench lanes. The first group touches no bench files, so it starts when a slot is free and no bench lane is waiting:

| Ticket | Why it can go early | Size |
|---|---|---|
| 0729 zero selection | Merge main, one gate. It may also cut build time on focused runs | S |
| 0649 Pi namespace | Land harness 9266ff73c, run the smoke once | S |
| 0640 document only | Refresh and land the document. The preflight stays held | S |
| 0712 build paths, 0713 container pins | Only `release/` files. This changes the settled order | S-M each |
| 0695, WHO half | The case and capture change together | S |
| 0641 part one | Restart from main: one Make goal and the test list. It sits ahead of 0293 by ruling, and it is the largest unstarted item | M-L in all |
| 0694 first step only | One job on main to learn whether the proofs are red, before anyone plans around it | job |

After the scored build lands, in this order where it matters:

1. 0725 after 0717. 0724 after 0718. 0693 if it has not landed.
2. 0723 after 0717, 0720, 0722 and 0693. 0676 after 0723, split by owner.
3. 0694, then 0711 with or after it. Both edit the same Makefile area.
4. 0670 fixes now that 0690 has landed, its Proof after 0641. 0695 Semantic Scholar half after 0717.
5. 0696. 0697 with its limits first, then 0714, because 0714 needs the higher limit. 0716 is withdrawn: the wheel keeps two copies for 1.0.
6. **0715 last**, after the bench and hosting commits, because every lane adds tests. It also clears internal names, which must be done before 0294.

Everything here beyond the hosting commit can follow first hosted validation.

## Wave 4: Ian's gates, then release

Ian and the PM evaluate sources. Ian decides whether 1.0 waits. Then 0640 preflight, 0293 rehearsal, 0294 public append, 0401 docs, in that order. No hosted deploy, preflight or cut happens without Ian.

- 0640: rows change as 0696, 0697, 0712, 0713 and 0716 land. The preflight is their Windows and Intel proof.
- 0293: 0649 and 0676 are ahead of it. Trim Depends when the W44 edit is applied.
- 0294: before the append, the internal names search is clean, 0695 has landed, and the rehearsal reruns on the final candidate.
- 0401: refresh once after the hosting commit, not before, and add a Node pin.

## Stop and ask the PM

A lane stops and asks before it:

- changes a store format, a cache namespace or the BioData pin,
- adds a source, or calls KEGG or the LitSense licensed subset,
- changes a hosted contract: a header, a startup flag, a tool name or a refusal hosting relies on,
- meets a rights question that no ruling covers,
- wants a new proof tool, a new gate or a per-ticket script,
- finds that a report example passes only if the fix is tuned to it,
- would build a local pool or a word list to make a source do what it cannot do.

## Rulings from the review

The PM ruled on 2026-11-06. The tickets carry these.

- 0717: "no query rewriting" is amended for source-derived drug names.
- 0689: a failed lookup keeps the abstract. The 20 record, 3 second bound stands, and the reply says what went unchecked.
- 0710: a study left out by policy leaves the reply complete, with a notice.
- 0721: counts, no percentage.
- 0722: land the paper list; the bench judges it. Europe PMC affiliations are a later ticket if needed.
- 0727: source names only, list as CLI JSON.
- 0726: source order plus a caller filter, under the no-rerankers ruling.
- 0723: generated check cut down; NCI input is stated, not resolved.
- 0724: five FAERS fields get words.
- 0712 and 0713 may start early. The rest of cleanup stays after the scored build.
- 0676: hosting's real off set first.
- 0716: withdrawn for 1.0.
- 0694: Python 3.13.
- Release tooling tickets land on local real-host proof; the preflight proves Windows and Intel.
- 0715 takes the new scope, including internal names. It must finish before 0294.
- 0696: marker in an HTML comment.
- 0728: proved by j32745, passed at its landed commit. The jobs its Proof named were cancelled.

Open with Ian: whether "support API keys" keeps OncoKB and AlphaGenome off at `restricted` even with a user's key. Nothing waits on it before hosted validation.

## Handover notes from the outgoing coordinator

- Lanes 0641, 0689, 0691, 0693, 0710, 0717 to 0722 and 0729 are pushed to their ticket branches, with next steps in `pm ticket progress`. Worktrees are kept.
- `wip/0689-withdrawn-store-scrub` is withdrawn. Never use it as a source.
- Run Make goals as `sh scripts/make_goal.sh`, because the file is not executable.
- Merges with source age need `data_date: None` in new `RequestGuidance` and `SyncResult`, and `regulatory_context: None` in new `DrugIndications` constructors.
- 0722's committed probe is unrun, and it goes away with the tooling drop.
