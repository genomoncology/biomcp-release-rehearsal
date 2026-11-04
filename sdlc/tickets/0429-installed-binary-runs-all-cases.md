# 0429 — Run the whole case suite against the installed release binary

Status: COMPLETE.
Landed: 4c48ea9d1ffde60c39bcee9c9057172b49dd52c0

Milestone: 5

Depends on: 0454

## Outcome

The release rehearsal runs every YAML case and every Rust trial in the `cases` target against the installed artifact on each release target, with `tests/` copied outside the checkout. The runner reads cases and fixtures only through `launch::fixtures()` and registers every trial whatever case directory it reads. The three installed proof modes become trials named by feature.

- Why: `release/smoke.py` runs one copied case and three special modes, so 653 cases never touch the artifact users install. `cases.rs` fixes its case directory at compile time, `--cases DIR` skips `proof::trials` and the protocol trials, and 26 test files read `CARGO_MANIFEST_DIR` directly, so a copied fixture tree is silently ignored.
- Starts from: `release/smoke.py`, `release/rehearse.py`, `release/gate.py`, `crates/biomcp/tests/cases.rs`, `runner/launch.rs`, `runner/proof.rs`, and the 26 test files that read `CARGO_MANIFEST_DIR`.
- Rules: `BIOMCP_TEST_EXECUTABLE` and `BIOMCP_TEST_FIXTURES` stay the only switches. A ratchet allows `CARGO_MANIFEST_DIR` in `crates/biomcp/tests` only in `runner/launch.rs`; integration targets in other crates (adapter, query and types tests) keep their own fixture and schema paths, because the installed rehearsal never runs them (m13614). The rehearsal timeout is the first measured run plus 15 percent. In-process trials that cannot run against an installed binary are listed for 0431 and not rewritten here. This ticket fixes fixture paths only. Any other failure on a platform, such as timing, a path separator or a missing system library, is filed as its own ticket with its raw log, and that platform's rehearsal marks those cases as expected failures named by ticket until the fix lands. 0293 and 0336 rerun their evidence after this lands.
- Proof: a planted installed binary that prints one changed schema ID fails the rehearsal, naming the case; a planted test file reading `CARGO_MANIFEST_DIR` fails the ratchet; the rehearsal reports the same case and trial count as the routine suite.
- Changes: the files above and `docs/development.md`.
- Defers: removing in-process trials (0431).
