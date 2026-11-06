# Plan: parity and release

This plan covers milestone 5. Every 1.0 entity slice has landed, so this milestone proves that 1.0 keeps what 0.9 users rely on, makes 1.0 safe to run as a long-lived server, documents it, builds every release artifact, and ends, after a blind comparison with 0.9.3 and Ian's approval, with the history append to the public repository, the 1.0 tag and publication. It names stages and the tickets that own them.

## Why this order

Robustness and public-text fixes come first, because the parity run, the docs and the rehearsal should all measure the code that will ship. The parity run comes before the docs rewrite, because any fact 1.0 loses against 0.9 becomes a fix that the docs must describe. The pipeline and schema gate come before the rehearsal, because the rehearsal installs what the pipeline builds. The public history append, tag and publication come last because they cannot be undone.

## Stages

### 1. Close milestone 4 and harden the server

- Finish the native concurrency diagnosis (0287).
- MCP server robustness (0283), store upgrades that keep bundles (0284, then coexisting generations in 0295), one-snapshot ownership checks (0296), replay miss reasons through one mapping (0285) and check hardening (0286).
- Port `version` and `mcp-config` (0288).
- Remove private infrastructure and vendor names from public text (0279).

### 2. Prove parity

- Run the harvested cases against 0.9.3 and 1.0 with a runner (0291) declared endpoints, a whole-case inventory and trial and pathway plans (0298), gene, article, variant, author and protein plans and the residual cases in parallel (0300 to 0305) on shared projection extensions (0306), multi-step and stateful execution (0299), then a partial recorded comparison, projection and completeness fixes (0307) and common sequence step shapes (0308) and a few comparable facts for each zero-coverage family (0309), then a whole run from main that fills the deferred size and timing comparison (0297). Each fact 1.0 loses becomes a bug ticket.
- Run the outside judged agent comparison against the latest released 0.9 (0098), PM-owned.
- Capture the LitSense gene search when the source allows it (0072).

### 3. Document

- Rewrite the README as a short guide, generate the reference docs and recipes (0292).
- Ship the agent skill guide (0187).

### 4. Build the release

- Carry the 0.9 release pipeline over, with publication gated on Ian's approval (0289).
- Compare schemas with the last released copy (0290).
- AlphaGenome: Ian approved the streaming gRPC transport. Build it in the shared fetch layer with zero provider calls: codec (0320), worker (0321), persistence (0322), then cost and target proof (0323), source and key registration (0333), the ScoreVariant encoder (0337); then the variant section with one clinically chosen capture (0198). It ships in 1.0, as `decisions/2026-10-27-1-0-ships-every-known-fix.md` says.

### 5. Close every known gap

- Docs findings, first pass: help entry points (0338), argument errors and grammar (0339), configuration and proxy (0340), schema IDs (0341), health (0342), output pipe (0343), help reference (0357), MCP output schemas (0358), schema formats (0359), adapter bad values (0360, then 0371 to 0375 by adapter family), enums and typed IDs (0361), output consistency (0362), live failures (0363), use case how-tos (0369).
- Docs findings, second pass: MCP config path (0352), HTTP MCP revisions (0353), openFDA generic names (0354), cBioPortal sync (0355), MCP refusals (0356), variant population (0364), GRCh37 normalize (0365), store size (0366), drugsfda items (0367).
- Security: screening (0347), inert source text (0348), outbound host registry (0349), signed release provenance (0370).
- Split-off and hygiene: PubMed warnings (0334), semantic argument names (0345), HGNC reachability (0368), build cost and check order (0344), export comment (0346), output budgets (0350), local fixture abort (0351).
- Rare disease rerun (experiment 499): variant transcripts and HGVS (0378), separate ClinGen statuses (0379), gnomAD from rsID or HGVS (0380), LitVar and variant article recall (0381), trial condition expansion (0382), GenCC section (0383).
- Order: schema-changing tickets (0341, then 0358, 0359, 0361, 0362) early, because each regenerates the baseline.

- Close the public benchmark gaps Ian added on 2026-10-28, corrected by an adversarial design review: build 0414, then 0409, 0411 and 0415 on the trial filters; 0408, then 0410 for the two NCI entities; 0413, then 0412 and 0416 for MAUDE filters and FDA device and drug recalls.

- Close the code quality review's findings (`records/2026-10-28-code-quality-review.md`) by the reviewed design in `planning/restructure.md`. Order: the trial status fix 0417, the reply diff 0454 that proves each structure ticket changes nothing it does not name, pure dead code 0425, the installed-binary suite 0429, then 0423, 0434, 0430, 0431, 0456, the bad-value rule (0467, then the families 0468 to 0472, closed by 0418), 0426, 0457, the native window owner 0494, refusal facts 0495, cache admission 0496, status rules 0419 and typed next steps 0458, the layer move 0420, the command table 0421, 0427, the source declaration 0422, 0455, 0424, 0428 and finally the architecture rewrite 0432. Each ticket's Depends line holds the exact order.

- Rebuild the test suite as one small outside-in suite (`decisions/2026-10-29-one-small-outside-in-suite.md`): the required-coverage check 0483 after 0431, then the case cut 0456, then the runner trial cut 0484 and the nightly selection 0485. Mutation testing 0457 runs on the cut suite.

- Make 1.0 ready to run as a shared hosted MCP server, live before 2026-11-30 (m16469): hosted mode 0502, then per-request keys 0503, then licensed replies kept per caller 0504 and per-key pacing 0505, and remote tool fit 0506 in parallel. These run alongside the restructure, because they touch the MCP edge and the fetch layer and serve the deadline.

- Fix the BioData review's data model findings before the tag, because they correct public values: 0435 to 0453, with 0436 and 0437 first, and 0444 before 0421.

### 6. Rehearse and release

- Test against public data before the cut: parse every CIViC and ClinVar record into 1.0 values (0460) and turn real user queries into cases (0461).
- Close the release review of 2026-10-31 (weighed 2026-11-01) and the code review tickets before the cut: bundle resume 0528, store failures 0529, the hosted HTTP edge 0530, compatibility options and small CLI fixes 0531, dependency trims and the unused-dependency check 0532, gated publication 0533, release notes and public docs 0534, and 0521 to 0526. They touch nothing in `crates/biodata`, so they run while the BioData switch waits. Job failures and gate time (0535) run alongside and gate nothing.
- Lift the hosted capacity limits the hosted team asked for on 2026-11-01: call and connection caps as settings (0548), keyed-only rate share (0549), shared TLS settings (0550), then published source pacing with config overrides (0551), and per-request declared use (0553).
- 0459 lists the code tickets that must land before the BioData switch, and 0336 lists those that must land before the cut. A new must-ship ticket goes on 0336's Depends line, or on 0459's when the switch needs it.
- Switch to the extracted BioData 1.0 crate once every code ticket lands, with an empty reply diff and an unchanged schema comparison (0459).
- Fix the agent benchmark's general causes and bugs before the cut (Ian, 2026-11-03): the generated front-door map 0586, MCP replies sent once 0587, every variant ID form 0588, bugs and errors 0589, lean default replies 0590. Experiment 513's ranked list may add more. Hosting's launch needs come first.
- Build the evals recommendations of 2026-11-04 (`planning/eval-recommendations-2026-11-04.md`) before the cut: drug names in `get drug` 0599, indel coordinates 0600, approvals newest first 0601, a map that pairs sections with sources 0602, and an empty-result note 0603. Main already uses one CLI-argument MCP tool and gives up on a cold download within the command budget. The Gaps list waits for evals to recheck it on main.
- Build the defects and cheapest high-value gaps from the evals gap recheck (`planning/eval-gap-recheck-2026-11-04.md`) before the cut: errors that name the filter and boolean values 0604, the openFDA label search page 0605, MAUDE received dates 0606, posted trial results 0607, and every openFDA label section 0608. Ian put the rest in 1.0 too (2026-11-04, m24381): FAERS several names 0609, gnomAD homozygotes and joint totals 0610, device approval search and documents 0611, cBioPortal rows 0613, GWAS studies by PMID 0614, CIViC level filters 0615, WHO PQ dates 0616, cell-line filters 0617.
- Cut the release candidate from main once every ticket above lands (0336).
- Ian put indication search and every finding of the 2026-11-05 hosted evaluation into 1.0 (`decisions/2026-11-05-everything-in-1-0.md`): indication search 0651, usage errors 0652, help 0653, hosted surface 0654, source status 0655, fetch text 0656, label ranking 0657, trial gene matches 0658, supplement indications 0659, variant joins 0660, drug code names 0661, FAERS counts 0662, gene full names 0663. Each release section passes on main before the next cut (0640).
- Before the cut, the landed fixes go live on hosting from main and are validated there. Then Ian and the PM evaluate what BioMCP agent testing finds and whether it needs new sources, and Ian decides whether 1.0 waits for them. 1.0 has no deadline; hosting on AWS and the agents framework come first (`decisions/2026-11-05-hosting-validates-before-1-0.md`).

- Rehearse the final candidate on every platform (0293), each section as its own job, rerunning only the section that fails (`decisions/2026-11-05-release-finds-no-bugs.md`).
- Run the multi-process cache checks under repeated load (0641).
- Comparisons with 0.9.3 and agent benchmarks belong to the evals team (experiment 513), not BioMCP; 0402, 0405, 0462 and 0490 were handed over (Ian, 2026-11-03, m23337 and m23350). BioMCP tickets the changes evals marks as sure wins.
- Ian approves BioData's public repository and its v1.0.0 tag, which 0459 pins (asked through Jarvis on 2026-11-01).
- Ian approves the publication credentials and settings 0533 lists. The release workflow has no manual approval pause (0554).
- Ian approves the public append, the 1.0 tag and publication.
- Append this repository's history to the public repository with one merge commit, keeping old tags and releases (0294).
- Tag v1.0.0 on that public merge commit, let the release workflow qualify the tag, and have the release operators publish from that run (0294). The installer and Homebrew download from the public repository's release, so the append and tag come before publication.

## After 1.0

Ian's direction of 2026-10-29 (relayed in m14819): BioMCP becomes the data and control plane for precision-medicine context for agents. It reaches FHIR servers and OMOP databases through adapters that turn their replies into BioData values, with no ORM and no hand-written API client. FHIR search and FHIRPath become BioMCP getters, and OHDSI's queries become BioMCP commands. None of this changes 1.0.

- BioData types only the small values: time with precision, decimal, unit, quantity, coded value, identifier, record key, source and loss (BioData's ruling of 2026-10-30, m15631). Record shapes, FHIR and OMOP adapters, SQL on FHIR views and query logic live in BioMCP, because FHIR R4 to R6 and OMOP 5.3 to 5.4 all stay live.
- FHIR commands: search and get patient with sections, search observation, condition, medication and encounter, get report, each planned from the server's capability statement with fixed fallbacks and echoed requests, and a sync command that fills an offline store through bulk export.
- OMOP commands: concept search and hierarchy, concept set resolution, person search and profile, Phenotype Library cohorts, counts and attrition, characteristics, incidence, and database summary and quality, running OHDSI's declarative artifacts (Circe, SqlRender, QueryLibrary, Achilles, DataQualityDashboard) natively in Rust, with OHDSI tools only as reference answers in tests.
- Open questions for planning: cohort counts break get study's promise to calculate no frequencies; real databases need a minimum cell count (OHDSI's default is 5); and whether linking a FHIR library counts as running another tool.
- BioData's experiments 507 and 508 tested shared clinical records and found that they failed across backends; planning starts from 508's record findings (encounter kind, observation stay, microbiology shape, medication kind).
- Licensed data stays private: GENIE, UMLS, TCGA, JAX-CKB and all other licensed material live only in private repositories, private experiments or our machines, and public ISCN tests share nothing with the ISCN book's examples.


- Build order for the remaining 1.0 tickets: `planning/build-plan.md`, with per-ticket detail in `records/2026-11-06-ticket-review.md`.
