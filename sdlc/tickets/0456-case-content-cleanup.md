# 0456 — Cut the cases to one best call per required item

Status: COMPLETE.
Landed: c9f7693a10b6e32f93619f5f22935d6f9ed1de6a

Milestone: 5

Depends on: 0483

## Outcome

`tests/cases/` holds about 400 command-line calls, one best call for each item on 0483's list, and 0483's pending list is empty. Duplicate cases are gone, the gaps have calls, and the routine suite runs in about a minute without coverage instrumentation.

- Why: Ian's ruling in `decisions/2026-10-29-one-small-outside-in-suite.md`. Of 680 case files with 1,756 steps, about 251 steps are needed. About 110 new calls fill the gaps: 49 refusals, about 47 option and section calls, 10 commands, 2 MCP tools and 1 global option. About 40 more add per-source failure checks for trials, variants, drug labels and articles.
- Starts from: 0483's per-item counts, `tests/cases/`, `tests/replies/`, and the runner trials that already launch the binary for a gap item, which can be ported to YAML.
- Rules: for each item, keep the call that checks answer values, uses real recorded replies, is live-checkable and tells a clinical story, in that order. Merge cases for one command into one file with several steps where that reads better. Synthetic replies are only for failures that cannot be triggered on purpose. MCP runs once per MCP tool and where MCP output differs, not as a repeat of every step. Delete unused recordings with their cases. Expected values are literals, never copied from product tables. Coverage drops as the duplicate cases go. After the cut, the PM sets the coverage floor from the new figure. A step whose successful output is plain text (`version`, `--version`, skill render) may assert exact stdout text or compare with an existing file, through the current runner, with no new runner or assertion language (m16104). The existing housekeeping runner trial that runs `version --verbose` and asserts its commit, target and path fields counts toward required coverage through the existing collector, registered once, because its output varies by build (m16107). The runner gains only three small step options: literal stdin (for `key set`), literal JSON assertions for known local commands that print native JSON without an envelope (`mcp-config`, skill install and status) while query commands keep envelope validation, and the exact usage error codes each advertised path really returns (such as `usage.schema`, `usage.unknown_command`, `usage.retired`) admitted by the collector (m16115). The case count is approximate: about 540 calls is fine, and no real contract is deleted to reach 400. The 0.9.3 parity stage is complete (0291 to 0309, 0297) and no open ticket uses it, so the cut deletes the parity runner, its plans in `tests/parity`, the Rust parity tests, `scripts/parity_runner.sh`, `scripts/capture_parity_inventory.sh`, `scripts/test_parity_*` and their make targets instead of keeping their inputs; the landed parity tickets keep their recorded results (m16319). A tools-only proof that reads a deleted case either reads a kept case or goes.
- Proof: 0483's check passes with an empty pending list. The case count and runtime are reported from a plain run. For a sample of ten deleted cases, a planted break of the deleted case's behavior fails a kept case. The reply diff is empty.
- Changes: `tests/cases/`, `tests/replies/`, 0483's pending list and `coverage.floor`.
- Defers: runner trials (0484), the nightly selection (0485).
