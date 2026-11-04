# 0454 — Compare full replies between two builds

Status: COMPLETE.
Landed: 34c2fa1c441f165fa3ddf329999a5d2c908c3edf

Milestone: 5

## Outcome

With `BIOMCP_TRANSCRIPT_DIR` set, the case runner writes one transcript per executed step, holding the argument list, the surface, the exit code and the exact stdout. `scripts/reply_diff.py BASE HEAD` compares two transcript sets and exits nonzero on any difference. Each structure ticket proves with it that its branch prints what its base prints, apart from the cases it names.

- Why: the restructure tickets move code without changing behavior, and cases assert chosen fields only, so a reordered step, a changed status or a changed message passes them. `decisions/2026-10-28-restructure-before-1-0.md` requires each ticket to keep existing behavior.
- Starts from: the step execution in `crates/biomcp/tests/runner/` and `runner/launch.rs`.
- Rules: a case step's key is its case path and step index. A trial step's key is the trial name, a hash of its argument list, and the count of earlier identical argument lists in that trial, so parallel spawns keep stable keys. The mask list starts with the build identity fields: the commit line of `version --verbose`, the executable path that `mcp-config` and help entries print, and the version string. Calibration runs two builds at different commits from different checkout paths with no source change between them, and each other differing field joins the mask with a comment saying why. Without the variable, the runner behaves as today. The diff runs as a `pm job`, never in the routine gate. The script stays under 150 lines. Ruling 8 leaves the tool's future to 0432.
- Scope (PM ruling 2026-10-28): capture covers every YAML CLI and MCP step and ordinary trial commands at their existing complete-output consumers, through a small capture wrapper and trial context. Transport and lifecycle trials, whose output a proxy would change, are not captured; the script prints the list of uncaptured trials, and a structure ticket that touches transport or process lifecycle relies on those trials' own assertions. 0432 decides whether that gap closes.
- Proof: the calibration pair diffs empty; a build with one changed byte in one case's stdout fails, naming that key and pointer; a build with one changed exit code fails; a build that changes one trial step's stdout fails, naming the trial.
- Changes: the runner step module, `scripts/reply_diff.py`, `docs/development.md`.
- Defers: nothing.
