# Ticket launchers submit the configured gate command unchanged

Status: OPEN.
Milestone: 5
Depends on: 0597

## Outcome

Ticket routine-gate launchers submit the exact `ticketsGate` command from `sdlc/pm.json` instead of keeping their own copies of it, and the redundant cargo-machete version-string assertion added by 0582 is removed.

- Why: a bounded root cause analysis (m24072) found four setup failures. Two of them, 0582 j28534 (a redundant machete version-string assertion) and 0597 j28855 (a missing configured Python prefix), share one cause: per-ticket launchers rebuild or bypass `ticketsGate`, while ordinary `pm job` runs execute only the command they are given and so don't inherit its Python prefix. The other two had separate causes (0597 j28848 a stale scanner baseline, 0596 j28568 a wrong runner selector) and are out of scope.
- Starts from: `ticketsGate` in `sdlc/pm.json`, the three existing ticket launchers the coordinator named in m24072, and the 0582 machete assertion.
- Rules: no new framework or global rule. Focused proof code and explicit scanner baselines stay unchanged. Any change to `sdlc/pm.json` goes through the gate.
- Proof: each launcher's submitted command equals `ticketsGate` byte for byte; one routine gate run through a corrected launcher passes. The PM checks one week after landing for zero new routine-gate setup failures from a missing prefix or a copied command.
- Changes: the three launchers and the 0582 assertion.
- Defers: nothing.
