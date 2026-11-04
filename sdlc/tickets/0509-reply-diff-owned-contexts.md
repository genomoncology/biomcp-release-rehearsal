# 0509 — Pair the remaining runtime values in the reply diff

Status: COMPLETE.
Landed: 91c086d7817b4e9fb11bb00bdac214f2280ac875

Milestone: 5

## Outcome

The reply diff pairs four more kinds of runtime value through 0501's existing helpers, each only where its owner is proven: acquired clocks bound per catalog root and epoch instead of across the whole reply set; declared opaque section and evidence continuation IDs with every reference to them; replay temp paths under an owned response-cases root; and exit codes, which become a compared output field instead of part of the alignment signature, so a command whose status and exit change on purpose still pairs and shows its exit difference. Any other difference still fails.

- Why: the 0419 paired comparison j23373 (m16794). Its normalizer exits 2 on equal head seconds that matched two different baseline clocks, on unpaired continuation IDs and temp paths, and on commands left unpaired only because their intended exit changed.
- Starts from: the 0501 normalizer, its tests and the j23373 raw archive on Yellow.
- Rules: pair by one-to-one binding inside the proven owner, never by a string mask. The diff derives each owner from the actual capture record and replay contexts; it takes no operator ownership map, and source or helper hashes never count as proof of ownership. A value whose owner cannot be derived stays unpaired and fails, and the developer reports it to the PM (m16827 ask 2). The 122 j23373 contexts whose replay owner cannot be derived stay failing in that historical capture; the capture records record and replay ownership metadata, and the 0419 final comparison recaptures only those owners with it, keeping the original evidence (m16827 ask 3); ID, reference and lifetime checks stay; no new framework, source build or whole capture rerun.
- Proof: the original j23373 counterexamples fail on the current normalizer and pass after; hostile negatives fail: a clock rebound across roots, a broken continuation reference, a temp path outside the owned root and an exit change on a command outside the declared scope.
- Changes: the reply diff normalizer and its tests.
- Defers: nothing.
