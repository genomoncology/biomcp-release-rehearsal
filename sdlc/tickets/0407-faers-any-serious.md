# 0407 — Restore the FAERS seriousness filters

Status: COMPLETE.
Landed: 5ecdbe8d5cd91e4da2ced65adac41bf77225b597

Milestone: 5

## Outcome

`search adverse-event --source faers --serious any` returns reports whose native report-level `serious` flag is 1. `--serious` also accepts `lifethreatening`, `disabling`, `congenital-anomaly` and `other`, each mapped exactly to its native openFDA flag (`seriousnesslifethreatening`, `seriousnessdisabling`, `seriousnesscongenitalanomali`, `seriousnessother`) equal to 1, alongside the existing `death` and `hospitalization`. 0.9.3 advertised all of these. Help says `any` uses the source's own seriousness flag and that the specific flags can overlap.

- Why: the 0405 benchmark replay (coordinator m12774, 2026-10-28) found agents trained on 0.4.6 filter FAERS with `serious=true`, and 0.9.3 also had a bare `--serious`. The adverse-event slice dropped the bare form only because its harvest had no such question. The native flag needs no inference.
- Starts from: the adverse-event slice ruling rows at `sdlc/planning/slices/adverse-event.md` 316 and 424, the FAERS adapter and its cases.
- Rules: map `any` exactly to openFDA `serious:1`; no counts, sums or causal wording. Aliases such as `hospitalisation` and `life-threatening` still fail before any request, naming the accepted spelling. One bounded live capture at most, within openFDA limits.
- Proof: replay cases show the exact query for `any` and each restored flag, with report headers carrying native codes; an alias still refuses; the slice ruling row and upgrade table are updated.
- Changes: the FAERS filter grammar, adapter, help, the slice row, the generated upgrade table and cases.
- Defers: nothing.
