# 0466 — Run the structure ratchets before the coverage suites

Status: COMPLETE.
Landed: f34c178bf223233b3ad55c5bcd1b2c75a6aa6c1d

Milestone: 5

## Outcome

The coverage run checks file sizes, placement and architecture rows right after the shared instrumented build, before the case and contract suites start, so a structure failure stops the run in minutes instead of after the full coverage collection.

- Why: four full runs (j18352, j18355, j20486, j20563) passed every case and contract suite and then failed only a structure ratchet. 0246 put the ratchets first, and the 0344 landing at 72a3e27a5 moved them back to the library step.
- Starts from: `scripts/coverage.sh`, where the ratchets run inside the library collection, and its command-order test in `scripts/test_coverage.py`.
- Rules: move the existing ratchet invocation; run it once per coverage run; keep its profiles in the excluded library report only, so it never counts toward the floor; add no new checker, framework or extra compilation.
- Proof: the command-order test shows the ratchet step runs after the shared build and before the case suite, and that a ratchet failure exits before any case or contract suite runs; the coverage figures on main stay unchanged.
- Changes: the coverage script and its test.
- Defers: reusing a completed proof across reruns, which belongs to the pm tool.
