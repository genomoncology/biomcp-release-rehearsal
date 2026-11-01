# 0534 — Write the 1.0 release notes and correct the public docs

Status: COMPLETE.
Landed: e038c6ebcc1269338ad49d219aa15d1eadc865f3

Milestone: 5

## Outcome

The release ships notes that tell 0.9 users what changed, the PyPI page carries the README, worked recipes use real public IDs with the question each answers, the README's own commands run as recorded cases, and public docs name no private tool.

- Why: release review C3, C8, C10, the C6 wording, and the README leftover of the 2026-10-27 product findings, weighed 2026-11-01. Ian's stance requires notes that list every removed or changed command, option, config key and output field. `release/workflow.py:138-151` writes only the attestation recipe, the FHIR line that `planning/milestones.md` claims exists nowhere, and the wheel metadata (`release/artifacts.py:87`) replaces the 0.9 PyPI page with one line. `docs/recipes.md:28,49,154,203` use synthetic IDs such as `semantic-scholar:123`, `synthetic` and `synthetic_0`. `docs/development.md:11,51` name the private job tool. README lines 69 and 73 repeat one sentence, and line 73 and `sdlc/planning/architecture.md:31` describe the schema check as blocking breaking changes, though it accepts any break that bumps the major version.
- Starts from: the generated upgrade table, `release/workflow.py`, `release/artifacts.py`, `docs/`, the README, `ROADMAP.md` and the cases with real public IDs.
- Rules: a hand-written notes source covers breaking changes (default JSON output, MCP wiring, removed commands and options), links the upgrade table, says FHIR patient reading is planned for 1.1, and lists schema IDs whose major rose. The PyPI description is the README with absolute links. Recipes are generated only from cases with real public IDs and each states its question. Add recorded cases for the README's `version`, `get gene`, `search article` and `search trial` examples. Carry `CITATION.cff`, `.zenodo.json`, `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md` over from the published repository with 1.0 facts, and replace `CHANGELOG.md` with a 1.0 entry (release review C4). Public text names no private tool, harness, vendor or project. Run the docs prose check from the shared skills.
- Proof: `make docs-check` passes and fails when a README example drifts; the built wheel's metadata shows the README; `rg` finds no synthetic ID in recipes and no private tool name in public docs.
- Changes: release notes source, release scripts, README, docs, ROADMAP, recipe generation and four cases.
- Defers: the docs site, which 0401 owns after publication.
