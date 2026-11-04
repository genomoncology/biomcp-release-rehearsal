# Job failures and gate time

Ticket 0535 cites this bounded analysis. Severity: Medium. The failures waste build time. This analysis does not establish a product defect.

## Observed cases and limits

The ticket reports 445 failures among its last 800 jobs and classifies 165 recent failures: 87 lint, compile, test or ratchet failures; 21 inline Python failures; 15 reply comparison failures; four unsupported Python archive calls; and about 32 native proof failures without an error line. The PM confirmed in m18218 that the 165-case classification came from an unretained one-off read of log tails. It is not a retained measurement. The source window and case list cannot be reproduced. The groups must not be treated as independently verified or disjoint.

A read of `pm job list --all` on 2026-11-01 returned 7,044 rows. Its newest 800 contained 442 failed, 310 passed, 40 cancelled, three refused and four waiting rows, plus one row whose position differed. This moving window does not reproduce the earlier review window. Refusals, cancellations and waiting jobs are not failed tests.

The retained logs establish these concrete cases:

| Job | Observed cause | Classification |
| --- | --- | --- |
| j24203, ticket 0432 | `TypeError: extractall() got an unexpected keyword argument 'filter'` | Proof setup |
| j24204, ticket 0432 | tools compilation failed before `scripts/trials.py --target tools --proofs --list` returned failure | Missing preflight |
| j24205, ticket 0517 | tools compilation failed with two diagnostics | Missing preflight |
| j24206, ticket 0517 | Clippy rejected a clone instead of `std::slice::from_ref` | Missing preflight |
| j24209, ticket 0432 | Passed; pm reports 575,046 ms | Successful gate with unmeasured stage costs |

`Makefile` invokes whole-history secret scanning on every landing. `scripts/coverage.sh run` explicitly cleans instrumented output before collecting profiles. The old gate has no stage clock. These source facts identify work to measure; they do not establish how much of the 575 seconds either stage consumed.

## Causes and smallest corrections

The repeated setup failures arise because launchers reconstruct proof recipes outside tracked targets and assume a Python API newer than the build machine. Use tracked make targets with explicit arguments. Use the installed platform tar when unpacking proof inputs. Keep Python 3.9 compatibility in shared scripts.

The compile failures arise because a proof discovers or compiles the tools harness before a common lint preflight runs. Provide one cheap pre-job target containing formatting, all-target Clippy and ratchets. It catches the concrete tools failures without collecting coverage or running release proof machinery. Clippy still has compilation cost; no seconds claim is established yet.

The successful gate repeats history work and hides stage costs. Print elapsed time and exit status for each stage, including failed stages. Landing may scan objects introduced since an explicitly selected previously scanned ancestor; release retains full history scanning. The initial `release/secrets-main` ancestor d527b7b031941dc290b2af00605c5baef8610715 is an ancestor of the source scanned by successful full-history gate j24209. Advance it only after a replacement ancestor passes screening. Reject invalid or unrelated baselines. Measure before changing coverage rebuild behavior.

Job request files contain private control information. Tracked `scripts/` and `release/` contain no job request copy in the searched Python and shell files. Copies from untracked launchers remain outside that finding. Any retained request copy must be created with mode 600 before bytes are written. Temp cleanup must inventory owners and ticket references before removing anything. A read-only inventory on 2026-11-01 found 1,366 BioMCP entries, including 613 `biomcp-test-*` folders. Recursive inspection found 31 retained `.log.request` copies, all mode 600. The random build-folder suffixes do not identify a closed ticket. The PM subsequently authorized deletion of the exact 613 scratch roots after a citation check. A read-only search of every current `sdlc/` file found no exact basename reference. The 613 roots contained zero file payload bytes and 29,409,280 logical directory-entry bytes including nested directories. Removal deleted those 613 roots and left zero `biomcp-test-*` roots. Directory-entry bytes do not measure recovered disk allocation. The PM owns 0405 and 0293 custody and requires all their temp entries to remain until those tickets close. Retained custody folders and all other analysis scripts remain protected.

The recipe and request handling causes also concern shared job tooling. The coordinator must route those findings through the PM to the shared rules or pm owner. This ticket changes no shared rules or pm code.

## Dated count checks

Ticket 0060 defines the Oct 26 check as coordinator asks per day and externally held lanes, the Oct 30 check as missing-license landing failures, and the Oct 31 checks as separate failure classes. `pm job list` does not expose dates or these classifications. Its failed-job total cannot answer those checks.

The ticket itself records minimum observed recurrences: two failed-step continuations, one useless test command, one source probe that misread an empty response, two reopened accepted shared type contracts, and at least three unsupported archive calls when its original two jobs and the later 0061 j14243 case are combined. These are lower bounds in the recorded historical window, not exact final daily counts. Unknown totals stay unknown. The PM confirmed in m18218 that no recorded source supports complete dated counts and recorded the Oct 26, Oct 30 and Oct 31 checks as not measured through pm. This analysis does not edit the 0060 ticket.

## Stopping rule

Implement the approved small fixes, run the affected checker tests and one routine gate, and hand the pushed branch to a fresh reviewer. Reuse retained evidence when its inputs still apply. Do not tune coverage before the stage measurements exist. Preserve open or cited evidence during cleanup.
