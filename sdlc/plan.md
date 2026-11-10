# Plan: finish BioMCP 1.0 through the remaining product issues

This plan covers milestone 5: a useful and accurate CLI/MCP interface, bounded and maintainable source handling, validated hosted operation, and an honestly qualified 1.0 release. Ticket state and lane ownership live in pm. The execution order and collision groups are in [build-plan.md](planning/build-plan.md); the [findings register](records/2026-11-10-product-review.md) records the reviewed snapshot and evidence, and the [test playbook](planning/remaining-issues-playbook.md) gives reproducible checks.

## Immediate priority

Ian's [November 10 direction](decisions/2026-11-10-prioritize-remaining-product-issues.md) puts finding and fixing remaining maintainability, usability and accuracy issues ahead of release timing and AWS publication. Review external claims as leads, inspect the code, independently weigh findings, then file small executable tickets. Do not call an unrun suspicion a reproduced bug or a later successful provider request an explanation of an earlier failure.

The review deliverable is the findings register, test playbook and ready fix tickets. Implementations follow in their owning lanes; no new product implementation is required merely to deliver this review.

## 1. Correct results and retained state

Preserve uncertainty in threshold totals; require complete evidence before expanding drug identities; keep incomplete stored observations from becoming current answers; and make pagination count native consumption separately from returned records. Fix the two cache behaviors previously deferred after publication. Repair shared causes at their actual owner while retaining source-specific behavior. Use controlled outside-in cases for malformed data, refusal, failed later pages and recovery.

A stored observation may remain explicitly addressable when it is incomplete, but cannot silently become the current answer for a new query. A healthy lazily paged observation need not download its entire source to be reusable. Raw transport caching has its own policy; do not conflate it with query-pool eligibility.

## 2. Correct hosting and operator behavior

Bound the freshness of cached issuer signing keys, preserve isolation between callers and test key removal with a controlled issuer. Retain useful diagnostics for each live-check attempt so intermittent failures can be investigated. Improve source-policy setup documentation without changing the open-by-default policy or removing prominent commercial-use instructions.

Review real CLI and MCP behavior with private temporary configuration and public keyless queries or synthetic fixtures. Hosted signed-in accounts, real key writes and deployment remain separately authorized actions. The local service health endpoint and upstream source probes answer different questions; provider timing failures need diagnosis, not repeated runs until green.

## 3. Review and land coherent fixes

Review each behavior change, then land compatible changes in batches with one normal gate per resulting candidate. Keep independently reproducible failing inputs and original failed reports. Add no new release framework, broad compatibility layer or generalized abstraction without a demonstrated need. Re-run only checks whose result could change after the accepted edits.

The existing source/licensing rulings, exact value and identity contracts, public command migration guide and one-source-of-truth architecture remain in force. A newly found issue gets a ticket or an explicit weighed disposition; unresolved observations get a named test rather than an invented fix.

## 4. Qualify the settled source and prepare publication

Select the next release source after accepted fixes settle. Keep earlier runtime, test-driver, package and image identities distinct. Reuse unchanged evidence where causal inputs match; refresh affected artifact, schema, platform, hosted and installed-channel checks. A new runtime change requires new runtime artifacts; a planning-only change does not justify rebuilding them.

Complete the existing ordinary rehearsal evidence verification and the three remaining manual installed checks: cleanup, the pinned 0.9 installer and the real Claude plugin. The cold AMD64 execution was emulated and must remain labelled that way. Native workflow proof is separate. Preserve the original failed mutation report, classified nonblocking survivors and failed native rehearsal packet.

After qualification and manual checks settle, refresh the history-append preparation and obtain its fresh review before assembling the publication proposal. Prepare the docs site before the cut against the selected public behavior. Avoid repeatedly preparing ancestry or previews while source inputs still change.

## 5. Validate hosting, then publish only with approval

When the rollout is the next useful step, gather the human inputs together: current AWS sign-in, confirmed PyPI Trusted Publisher settings, the concrete rollout/account setup approval and signed-audit client inputs. Do not request expiring device codes while review work is still in front of the rollout. The two signed client sessions and genuine expired token must be ready before starting an approved deployment whose validation depends on them.

Refresh AWS/account/task/image guards immediately before approved writes. Deploy the exact reviewed image mapping with immutable tags and the false-Docker guard; do not rebuild during deployment. Complete the live signed audit, client/capacity checks and source assessment under the existing AWS-first ruling. New source scope is Ian's decision.

For publication, prepare one local merge with current public main as first parent and the approved final candidate as second, preserving old history and tags. After explicit approval, push only the `v1.0.0` tag on that merge. Its ordinary workflow qualifies, publishes, checks public-channel installs and promotes. Fast-forward public main only after success; failure leaves public main untouched. Land the prepared docs and publish the site in that release session under the relevant approval. The history-append ticket owns this sequence. There is no promised release date.

## After 1.0

Ian's direction of 2026-10-29 (relayed in m14819): BioMCP becomes the data and control plane for precision-medicine context for agents. It reaches FHIR servers and OMOP databases through adapters that turn their replies into BioData values, with no ORM and no hand-written API client. FHIR search and FHIRPath become BioMCP getters, and OHDSI's queries become BioMCP commands. None of this changes 1.0.

- BioData types only the small values: time with precision, decimal, unit, quantity, coded value, identifier, record key, source and loss (BioData's ruling of 2026-10-30, m15631). Record shapes, FHIR and OMOP adapters, SQL on FHIR views and query logic live in BioMCP, because FHIR R4 to R6 and OMOP 5.3 to 5.4 all stay live.
- FHIR commands: search and get patient with sections, search observation, condition, medication and encounter, get report, each planned from the server's capability statement with fixed fallbacks and echoed requests, and a sync command that fills an offline store through bulk export.
- OMOP commands: concept search and hierarchy, concept set resolution, person search and profile, Phenotype Library cohorts, counts and attrition, characteristics, incidence, and database summary and quality, running OHDSI's declarative artifacts (Circe, SqlRender, QueryLibrary, Achilles, DataQualityDashboard) natively in Rust, with OHDSI tools only as reference answers in tests.
- Open questions for planning: cohort counts break get study's promise to calculate no frequencies; real databases need a minimum cell count (OHDSI's default is 5); and whether linking a FHIR library counts as running another tool.
- BioData's experiments 507 and 508 tested shared clinical records and found that they failed across backends; planning starts from 508's record findings (encounter kind, observation stay, microbiology shape, medication kind).
- Licensed data stays private: GENIE, UMLS, TCGA, JAX-CKB and all other licensed material live only in private repositories, private experiments or our machines, and public ISCN tests share nothing with the ISCN book's examples.
