# 0297 — Run the whole parity comparison and fill the size and timing rows

Status: COMPLETE. Correctness accepted (114 comparable, 0 regressions) and matched-release measurement j16492 accepted: 29.8 percent smaller output, no general speed claim. Recorded in decisions/2026-10-27-parity-result-accepted.md.

Milestone: 5

Depends on: 0291, 0298, 0299, 0300, 0301, 0302, 0303, 0304, 0305, 0306

## Outcome

A build delegate runs the landed parity runner from main over every harvested case against released 0.9.3 and the 1.0 candidate, records the per-slice size and timing rows, and files a bug ticket for each fact 0.9.3 got right and 1.0 gets wrong.

- Why: the milestone 5 parity result and the size and timing comparison every slice deferred come from one whole run on landed code. PM ruling 2026-10-26 (m8126 ask 27).
- Starts from: the landed 0291 runner and make target, the 0.9.3 release binary and main at the time of the run.
- Rules: PM ruling 2026-10-27 (m8126 ask 46): the run happens in two passes. The first is an explicitly partial recorded comparison now over every case and step that is executable, which reports all gaps and files a bug ticket per regression; it makes no parity claim. The second is the whole run after 0307 and the selected 0308 shapes land and the PM has judged the held gaps, including native recording gaps, in one table. Formal timing uses comparable release builds of both binaries on one machine. Released 0.9.3 is the baseline. A fact 0.9.3 got right and 1.0 gets wrong becomes its own bug ticket. A fact 0.9.3 got wrong is reported, not counted against 1.0. Live calls stay bounded and keyless or name-inherited, and no key value is recorded.
- Proof: the partial pass report lists every executed case with both verdicts, sizes and times and every gap by kind. The whole pass (j16465) accounts for every harvested case and step and lists every comparable fact with both verdicts, every assessment gap and every unsupported fact; it is not required to be gap-free (PM ruling 2026-10-27, m8126 asks 47 and 48). A reviewer checks a sample of verdicts against the raw outputs, and every regression has a ticket. The PM judges parity over comparable facts with the full gap table after 0309 and the gene-sample supplement.
- Changes: the per-slice size and timing rows in ROADMAP.md and the slice docs, and new bug tickets.
- Defers: the outside judged agent comparison, which is 0098.
