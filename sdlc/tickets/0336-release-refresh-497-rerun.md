# 0336 — Cut the 1.0 release candidate from main once every 1.0 ticket lands

Status: OPEN.

Milestone: 5

Depends on: 0459, 0547, 0521, 0522, 0523, 0524, 0525, 0526, 0528, 0529, 0530, 0531, 0532, 0533, 0534, 0548, 0549, 0550, 0551, 0553, 0554, 0555, 0556, 0557, 0558

## Outcome

`release/1.0` takes main's tree in one merge commit once every listed ticket has landed on main and the schema baseline has been re-exported on main as its last commit before the merge, and one approved hosted release.yml run passes on the new head.

- Why: `decisions/2026-10-27-1-0-ships-every-known-fix.md`. Ian wants every known fix in 1.0, so the candidate is cut from main instead of cherry-picking chosen commits.
- Starts from: the `release/1.0` head after 0332, main after the last listed ticket lands, and the 0314 snapshot recipe.
- Rules: the merge has `release/1.0` as first parent and main as second, and its tree equals main's. The manifest names the real product source commit. The hosted run on the new head is the installed proof. The earlier cherry-pick plan and the m10484 net-commit ruling are superseded, because main's whole tree now ships.
- Baseline (PM ruling 2026-10-28, pre-release review): re-export `release/schema-baseline.*` on main from main's own source as the last commit before the merge, with its status naming the 1.0 release candidate, so the merge tree still equals main's tree and 1.0.1 compares against the shipped set.
- Release comparison (release review C7, 2026-11-01): the 1.0 schema comparison runs against the baseline this ticket exports, so it compares 1.0 with itself; the first real comparison is 1.0.1's. A break that bumps a schema ID's major version passes, and the release notes list it.
- Proof: the merge tree equals main's tree; the gate passes; the baseline equals the release head's generated schemas; release/1.0 holds no conflict markers; the hosted run passes all five targets, containers and Homebrew.
- Changes: the `release/1.0` branch and its baseline.
- Defers: publication and the history append, which need Ian.
