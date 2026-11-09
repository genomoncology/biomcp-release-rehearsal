# 0640 — Release sections pass on main before the cut

Status: COMPLETE.
Landed: deca3535b4622a6f0405525182094ea78ed79cee

Milestone: 5

## Outcome

`release/qualification.md` lists every release section with the one command that runs it alone, its host and its inputs. Before the next candidate is cut, each section has passed alone against a main commit on its real host, so qualification starts from sections already known to work.

- Why: `decisions/2026-11-05-release-finds-no-bugs.md`.
- Starts from: `release/qualification.md` and its existing commands.
- Proof: retain each section's source, runner, host, command, exit and outputs; review the combined evidence before closing this ticket.
- Changes: the qualification document and this evidence record. No production code or packaging changes.

## Selected product and retained evidence

The release product remains `b3809eb1baa6d1e470efaf6499d8be0ba600f511`, tree `4baf7bddd9487d5adba82589c45351bef36fa2b2`. Evidence-only commits do not select new artifacts. The normal public rehearsal is [42680531044](https://github.com/genomoncology/biomcp-release-rehearsal/actions/runs/42680531044).

The receiver-verified evidence is on Yellow:

- `data/biomcp-redux-evidence/0640/b3809eb1baa6/`: `j36350-receipt.json` binds the bounded mutation results; `j36356-cold-receipt.json` binds the cold outputs.
- `data/biomcp-redux-evidence/0293/b3809eb1baa6/run-42680531044/`: `receiver-manifest.json` binds the actual API records, raw logs, downloaded artifact ZIPs and extracted payloads; `provenance-helper-receiver-manifest.json` binds signature and local-channel records. `download-receipt.json` records every skipped production step.
- The full suite, schema comparison and load evidence remain under `data/biomcp-redux-evidence/0640/42a79b1bbdc9/`, with their original identities. 0778 changed only Fetch integration tests, mutation selection and its tests, and the ticket. Runtime, Cargo pins, platform producers, installed cases, fixtures and workflow bytes were unchanged. The ending mutation baseline exercised both CLI cases and the revised Fetch cache tests.

| Section | Actual result and scope |
| --- | --- |
| Full suite and schema comparison | `j36287` passed all eight stages and 740 cases at `42a79b1bbdc95108447d19369b7cd69f35f8adef`. Ending native Linux qualification also passed the schema comparison at `b3809eb1baa6d1e470efaf6499d8be0ba600f511`. |
| Cache and hosted load | `j36288` and `j36305` passed the ten selected affected checks, including 400 successful fresh macOS cache opens; 300 hosted calls with eight real local publication windows; and both hosted surface trials. These are the stated subset at `42a79b1bbdc9`, not another whole stress run. |
| Bounded mutation collection | `j36350` passed at the selected product: 29 of 1,039 mutants selected and completed; 21 caught, eight unviable, no survivors, timeouts or errors. The budget left 1,010 unrun. Baseline `2d2dbda17ca29202bd332152ab604408cb9b5b14` keeps cache.rs in scope. CLI cases and Fetch cache tests ran together; one proven disjoint-flags OR-to-XOR equivalent was excluded consistently. The retained complete native calibration and actual plan, shard and collection remain in the packet. |
| Cold Linux AMD64 | `j36356` passed all ten inner and eleven outer stages at the selected product, using a full nonshallow clone and empty container HOME, Cargo and Rustup state. Five installed channels ran 740 cases each. This is AMD64 **emulated on ARM64 M5**, not native Linux proof. The actual native Linux x86_64 and ARM64 qualifications passed in the ending public rehearsal. |
| Cold Linux ARM64 carryover | At `42a79b1bbdc95108447d19369b7cd69f35f8adef`, `j36290` passed its actual ARM64 build, schema, all five installed channels (740 cases each) and fixed-runtime container (740 cases and 28 proofs). Its original compound result stays **failed**, exit 2: the history scan could not read the host-linked worktree Git directory inside the container. Scanner-only `j36327` then passed from a full clone at the same source, on both container architectures. The 0778 delta changes no ARM64 production, compiler, runtime package, cold recipe, installed runner or fixture inputs; this is retained cold-environment proof under its original identity. Ending native ARM64 artifacts were separately built and qualified at `b3809eb1baa6` by the ordinary workflow. Receipts remain under `0640/42a79b1bbdc9/j36290-cold-receipt.json` and `j36327-history-receipt.json`. |
| Native installations | The ending workflow passed all five native targets. macOS ARM64 and Intel each passed six channels with 740 cases per channel; Linux x86_64 and ARM64 each passed five with 740; Windows x86_64 passed five with 734. Homebrew was deferred on Linux and Windows and actually passed on both native macOS targets. |
| Runtime containers | Both ending native Linux container gates passed. The cold emulated AMD64 container also passed 740 cases and 28 runtime proofs, with the Python-dependent proof explicitly excluded. Fixed zlib package versions and archive receipts were checked against the manifest. |
| Stage, screening and signatures | The ending workflow prepared and screened the complete payload. All 26 staged regular files matched the subject inventory and independently verified signed provenance for the exact repository, workflow, source ref, selected product and run. `provenance-verification-receipt.json` records them. |
| Staged Homebrew, wheels and installer bytes | Both native staged Homebrew jobs and the staged-payload job passed. The latter ran strict wheel metadata validation and compared the staged installer bytes. |
| Pinned 0.9 installer and real Claude plugin | `j36486` passed on native macOS ARM64 using the ending archive and binary. The pinned old installer downloaded only from the helper's local server. Claude installed the staged plugin in an isolated HOME and executed its MCP help launch without invoking a model or a source API. |

## Installed cleanup and documentation validation

`j36488` passed the existing installed cleanup helper on native macOS ARM64. It installed the genuine ending wheel (binary SHA-256 `b577062c27545d2501d38658f3acab96195c2646654a4edcd89c7129a2d513ad`) and used matching product, runner and tooling source `b3809eb1baa6d1e470efaf6499d8be0ba600f511`. It ran 740 cases and asserted successful and failed temporary-root removal and preservation of the job-owned sentinel. `result.json` records exit 0 and no observation errors. `binding.json` binds actual runner hashes, fixtures and helper sources; `cleanup-helper-receiver-manifest.json` records receiver verification of the raw helper output and retained native macOS runners.

The host row now describes actual portable Linux/macOS behavior. `scripts/installed_smoke_section.py:21-37` treats missing Linux `/proc` entries as no supplementary process observations; `release/test_smoke.py:142-157` makes the success/failure cleanup and sentinel assertions. macOS does not claim Linux process observation. No Linux rebuild was needed to execute these portable assertions.

`j36499` passed `make docs-check` at documentation revision `4287502a64a02be9f091185d0d87a92bc7f77aa5`. Its qualification document is byte-identical to this branch; subsequent changes only record ticket evidence. The packet's `docs-check-receiver-manifest.json` binds the actual command, result and full log.

## Boundaries

Rehearsal publication, channel authentication, public-channel installs and latest promotion were skipped. A green no-op channel job proves dispatch reached its boundary, not that an external service accepted a package. This ticket authorizes no public tag, publication, account change or hosted deployment. Earlier failures retain their original source identities and are not relabelled as the ending product.
