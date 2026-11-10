# Nine fix implementation and validation

November 10, 2026. All nine fixes from the approved implementation plan landed. The core runtime producer is `972c98e98f483d52a7cbd76996a6db422282dd5d` (tree `13e6851930c5cb066b57f18457025ce55444c613`); hosting landed at `ca62d2de456b3e56ffcfe3d76352da7b4f8bee63`. Ticket and release state remain in pm. This report records the implementation and its evidence, not approval to publish or deploy.

## Resulting behavior

- GWAS threshold searches retain uncertainty across pages and older saved replies. Incomparable or refused records cannot produce a complete exact qualifying total; equality, true zero and underflow controls remain correct.
- Incomplete source pools remain explicitly addressable but do not become defaults for new searches. Separate current keys preserve old identities, consistent reads authorize metadata before payload access, and accepted updates change payload/current eligibility atomically. Healthy lazy windows and local study catalogs remain reusable.
- Drug-name expansion requires complete candidate evidence, retains bounded fallbacks, and preserves incomplete resolution in article admission. Legacy implicit expansions cannot bypass the corrected resolver; explicit pages retain their identities.
- Trial continuation offsets count native consumption while returned counts describe admitted rows.
- Narrow cache readers skip reusable oversized entries without deleting them. Corrupt entries still evict and oversized stale data cannot answer a narrow reader.
- Expired offline absence retains the Offline error with fixed historical-uncertainty guidance; it never becomes current absence and makes no network call.
- Hosting signing-key freshness defaults to 300 seconds through both configuration readers. Refresh coalesces, attempts are throttled even after cancellation, expired refresh failure gives 503, and a successfully removed signing key gives 401. Both authentication surfaces share the policy.
- Nightly diagnostics retain flushed, bounded attempt records without source bodies, arbitrary notices, headers, credentials or malformed output. Retry and final exit policies remain intact.
- Source-policy setup has one complete configuration home with a prominent README link. The checker reads the canonical example and retains negative tests.

## Gates and targeted checks

Core gate **j36792 passed all 754 cases and 47 structural checks** on its actual ending commit. The eight fixes were reviewed individually and landed as one batch. The gate includes the six relevant protocol/caller cases: typed CLI bridge, reply forms, typed followups, header boundaries, continuation rights and cancellation preserving another call. They passed, including the changed ORCID fixture; a suspected missing volatile projection was not a reproduced defect.

Hosting gate **j36727 passed** on its actual ending commit: 126 infrastructure tests, five library and one binary unit test, and 50 outside-in tests. One pre-existing ignored test remains explicit. No scanner rules, dependencies or runtime policy were widened to repair test failures.

Post-landing **j36803** additionally passed health (1 test), transport cancellation (2 tests) and credential-cache isolation (3 tests). Its initial proof-selector stage selected the wrong runner and is a setup failure, not extra protocol proof. The actual passing protocol evidence is j36792. Its exploratory driver is separate from the genuine product binary.

Preserved failures include actual baseline regressions, fixture/compiler setup failures, two early core structural stops, an obsolete ORCID implicit-reuse assertion, and the first hosting fixture-isolation gate failure. The ORCID correction strengthens the contract: it follows an actual returned continuation, retains partial diagnostics on explicit offline pages, and requires Failed/Offline for a new implicit offline query. A source-test declaration was aligned and wrapped without changing allowed paths or checks. Hosting tests now use ephemeral in-memory signing keys and prove valid same-key-ID replacement.

## Live-source diagnosis

Each of the three previously failing commands ran once with the genuine ending M5 ARM64 debug binary. Both Cellosaurus commands returned complete. Article search `AKT1 E17K` returned partial with PubMed OK and ChEMBL/openFDA-label deadline diagnostics.

One separately authorized observed call, **j36821**, explains that pattern on this source: MyChem completed HTTP 200 in 292 ms; ChEMBL entered transport after quota admission and consumed 6,372 ms without a completed HTTP status; their combined time exhausted the approximately 6.67-second shared drug-name budget. OpenFDA made zero transport attempts. PubMed completed and returned 193 identities. There was no retry.

This establishes shared-budget exhaustion and rules out local quota waiting for the measured ChEMBL exchange. The precise DNS/connection/response/body delay inside that exchange remains unresolved; no provider outage or TLS cause was proved. It does not retroactively establish every detail of the earlier nightly failure.

Independent weighing found the current partial typed-term result and generic notice satisfy the bounded-lookup contract. A source-specific explanation for an unattempted fallback is recorded as idea 0801, triggered when resolver diagnostics are next changed. It is a Low clarity improvement, not another correctness or qualification blocker. No heuristic, status, budget or provider policy was changed to obtain a green live result.

## Documentation preparation

The prepared docs branch is freshly accepted at `021736834930825b6c8af21f4149dfa2b889e183`, merging the ending core and preserving its existing four-line test-only help exporter. It includes the source-policy changes and corrects the GWAS caution so an exhausted walk alone does not promise an exact total.

**j36819 passed docs-check and site-build** with Python 3.13 properly selected for child make commands: 20 exact examples, 440 reference files, 527 checked pages, 6,654 anchors, 137 redirects, 201 old addresses, and positive/missing-installer tests. No generated drift occurred. The preview's source and program are both the actual docs driver; schemas match core. The 197 saved examples include 18 unverified entries; this job performed zero human/private-account checks and no live capture refresh.

Earlier post-test docs attempts failed because an investigation-only script contained a private path, then because make selected system Python 3.9. Their outputs were preserved. A duplicate retry was cancelled once the correctly pinned docs job covered the missing stages. These are setup outcomes, not product documentation failures. The site remains prepared for its separately approved release-session landing and publication.

## Retained evidence

All paths below are on Yellow under `data/` and were receiver-verified.

| Evidence | Location | Manifest SHA-256 |
| --- | --- | --- |
| Core ending receipt, six payloads | `biomcp-redux-evidence/0785-0798/core-batch-20261110/final-receipt` | `49e59645c0f54415265e2f5af460f0059df9b2665a40f324cd7c7b14c520bfe7` |
| Genuine ending CLI | `biomcp-redux-evidence/0785-0798/core-batch-20261110/gate-j36792-artifact` | Executable SHA `0fa14f7fc0aba554d9b2b759b482732a8d476eae555a6ff66d495dfccf966fa8` |
| Hosting ending receipt, 15 payloads | `biomcp-hosting/evidence/0091/landed-ca62d2de4` | `614bd3b22bd68a6e85ba478ac555f54a8a08421b22ebb599ca9a42afc45b3cad` |
| Observed article call, four payloads | `biomcp-redux-evidence/0788/article-observe-20261110` | `fe970ec9a0d4ddcd2a0169e28a0f5c76633f402e8e996cf111f9df8e7ea0b872` |
| Prepared docs, nine payloads | `biomcp-redux-evidence/0401/docs-972c98e98-021736834` | `27c3f997a7642b7a5540a62fe48b6ac517dad92efc7eae0664d932b906ec43e9` |

Post-test raw logs are separately retained at `biomcp-redux-evidence/0788/postlanding-j36803`, `postlanding-j36812` and `postlanding-j36822`; the cancelled run stays labelled cancelled. Product, docs-driver and investigation-driver identities are distinct. Failed-candidate executables are not relabelled as the ending source.

## Remaining release work

This completes the nine-fix implementation and targeted validation. It does not supply a new all-platform release rehearsal, full release mutation/artifact matrix, signed-in hosted audit, deployment or public publication. Earlier 0ad/a84 results retain their original sources. The changed runtime needs the affected final qualification and hosting image/pin preparation before a concrete rollout/publication proposal. Owned ticket/red-proof worktrees and branches were cleaned after archival; the open docs and deployment work remain preserved.
