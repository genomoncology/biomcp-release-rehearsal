# 0518 — Register the team's data folders

Status: COMPLETE.
Landed: 25f558ac16251c4e910483ef48989623f8976f53

Milestone: 5

## Outcome

Every data folder this team owns on any machine has one entry in `sdlc/data/NAME.md`, and `pm disk` lists none of them as unregistered.

- Why: the shared data-location rule makes each repo's `sdlc/data/` entries the one record of data folders on every machine, and this repo has none. The lander of the 0432 and 0517 batch found its verified archive and the internal hosted build with no entry (m17770 ask 2).
- Starts from: `pm disk`, the shared rule in the PM skill and digest, Yellow's data README, and the folders named `biomcp-redux`, `biomcp-redux-evidence`, `biomcp-redux-0410-native` and `biomcp` under each machine's `data/`.
- Rules: write one entry per asset with Owner, Role, `Location: MACHINE PATH` relative to the machine's workspace root, `Copy:` lines for verified copies only, and `Kind:` and `Get again:` where known. Name each entry's owning ticket. Read only listings, manifests and receipts; never call two folders copies from size or path. Delete, move and copy nothing. A folder with no known owner is listed in the reply to the PM, not guessed. Ian's data/ian is out of scope.
- Proof: `pm lint` passes, and `pm disk` shows no unregistered folder of this team on any reachable machine.
- Changes: `sdlc/data/` only.
- Defers: cleanup of leftover builds, which follows the existing cleanup rules.
