# The release run finds no bugs

Ian, 2026-11-05: "The release process [should not] find any bugs. All the bugs should be found before the release process, but then what do we need to do to speed up the release process? Is there a specific section of it, like Windows or something? If so, just run that particular part till it's done. Get us a dedicated action and run just the dedicated action."

## What it means

- Release qualification confirms a candidate. It is not where defects are first found. Each release section runs alone against main, on its real host, and passes before a candidate is cut (0640).
- Each section has one command that runs it alone. When a section fails, only that section reruns. A fix that touches only tooling reruns its own section, without a new cut or reruns of the other sections. A product source change still means a new cut, as 0293 says.
- A section that keeps failing runs by itself until it passes, apart from the rest.
- PM application: a ticket that changes a release section's tooling is complete only when that section has passed end to end on its real host. 0635 landed without its Windows VM run, which left the wrapper unproven at the cut.

## Reason

Since the 1.0 candidate of 2026-11-04, qualification found nine problems, one run at a time. Eight were in release-only tooling, the history scan's allowlist and test timing. One was a product bug that a concurrency test missed because it runs only once (`records/2026-11-05-release-finds-one-at-a-time.md`).

## Options considered

- Keep requalifying every section on each candidate. That is how the nine finds arrived one run at a time.
- Accept the hosted Windows job, which runs the same `release/gate.py` installed gate, and drop the local Windows VM run. First not chosen, because Ian asked for the failing section to run until done. Then chosen for 1.0 (PM, m26377): the VM wrapper reached the installed cases (j30357), and its next failure was a VM fault, a freshly vendored file vanishing mid-build (j30533), with no product cause. The hosted exact-release Windows run is the Windows proof. The VM run continues outside the release path once Jarvis has looked at the VM.

## Replaces

Nothing. It adds to `decisions/2026-11-02-public-rehearsals.md` and to 0293's carry-over rules.

## Amendment, 2026-11-06

PM ruling: 0640 needs release tooling tickets landed before it can run, so a tooling ticket may land on proof from a local real host. The 0640 preflight is its Windows and Intel proof, and a failure there reopens the ticket that caused it. Defects are still found before the cut.
