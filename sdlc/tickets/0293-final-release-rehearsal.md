# 0293 — Rehearse the final release candidate on every platform

Status: OPEN. Reopened: 0336 adds 0334 and 0335 to the candidate; the final rehearsal must qualify the refreshed release head (m9956).

Milestone: 5

Depends on: 0289, 0290, 0292, 0297, 0319, 0332, 0336, 0621, 0622, 0634, 0635, 0636, 0638, 0639, 0640, 0651, 0652, 0653, 0654, 0655, 0656, 0657, 0658, 0659, 0660, 0661, 0662, 0663, 0665, 0666, 0667, 0668, 0669

Reopened again 2026-11-05: after 0636 merges typed mode into main, re-cut the candidate from that main (0336's refresh steps, including the schema baseline export, keeping the immutable migration baseline) and qualify the new head in full: the hosted rehearsal on all five platforms and the local cold matrix, since product code changed. 0633, 0634 and 0635 tooling carries over.

Reopened 2026-11-05 (m26035): installed candidate 015aea0a reproduced a user-reachable cache startup `Busy` at the `journal_mode=WAL` switch (j30327, 1 of 24 concurrent starts on a fresh cache). After 0639 fixes the cause, cut one final candidate from main with 0639 and 0638's tooling and qualify it in full; 015's results do not carry over for product proof.

Reopened 2026-11-05 (`decisions/2026-11-05-everything-in-1-0.md`): Ian put indication search and the hosted evaluation's findings into 1.0, so 1bb97e64f is not the final candidate. Its section runs stand as tooling proof only. After 0640's preflight passes on main with those tickets landed, cut one candidate and qualify it section by section.

## Outcome

The final release candidate's artifacts install and run on every promised platform, repeating the 0066 installed rehearsal: wheel, installer archives, Homebrew formula, container, desktop bundle and registry entry, each running `version`, `health`, one replayed search and get, and stdio and HTTP MCP. The report names any platform that could not be exercised.

- Why: the release plan requires final-candidate installed proof before Ian approves publication.
- Starts from: 0066's rehearsal recipe, `release/gate.py` and `release/matrix.sh`, release.md and the machine registry.
- Rules, rewritten by PM ruling 2026-10-28 after the pre-release review: the candidate is the `release/1.0` head that 0336 produces, which carries main's whole tree. Earlier candidates' evidence does not carry forward, because product source changed. The 0336 hosted no-publication run on standard free runners is the cross-platform build proof, under Ian's standing approval for such runs. The rehearsal also exercises the Claude Code plugin channel and the installed AlphaGenome local TLS proof from 0392 on the five native targets and both Linux containers. The schema baseline is the one 0336 re-exported. Machines are booked through pm job, and nothing is published.
- Proof: one report per platform with exact commands, versions and exits, reviewed by a fresh reviewer.
- Changes: none expected beyond fixes the rehearsal finds, each its own ticket.
- Defers: publication, which needs Ian.

- Release review (2026-11-01): the rehearsal also runs the published 0.9 installer script against the 1.0 archives, and resolves or classifies the amd64 Docker containerd ENOENT fault before it passes. The evidence below is history from earlier candidates and proves nothing about the final one.

## Evidence

The qualified release candidate is `release/1.0` at `ff2e9b64c8b6f205ecd60c3bc51874bfa7223b55`. Its product source is `e4b72e59f590f4b04189718d8ecdd633e4238307`. The refreshed baseline contains 275 schemas generated from that exact source. Local job `j17367` passed baseline equality, schema comparison, the six fixes, container checks and the routine gate.

[Hosted run 40542150253](https://github.com/genomoncology/biomcp-redux/actions/runs/40542150253) qualified that exact release candidate through `workflow_dispatch`, attempt 1. The repository was public, and every runner label was a standard free hosted runner. All five native platform jobs passed: Linux x86_64, Linux arm64, macOS Intel, macOS arm64 and Windows x86_64. Both Linux container checks passed. Each platform's retained install report records version, isolated offline health with all 56 source IDs disabled, replayed search and get, and stdio and HTTP MCP checks for the installed channels. Both macOS platforms passed actual Homebrew installation. Homebrew was deferred on Linux and Windows. The Linux x86_64 schema step and publication payload staging passed. The release operator job was skipped. Nothing was published.

The raw evidence is retained on Yellow at `/home/ian/workspace/data/biomcp-redux/0293-release-rehearsal-evidence/candidate-ff2e9b64c8b6/hosted-40542150253`. Its 132-file manifest has SHA-256 `0abc92aaebe5de95ad61504ac89256621c574688c5620f7fabf33e7d062b15b3`. The archive holds terminal API records, raw logs, exact workflow and tooling, five platform install reports, artifact download receipts, member hashes and binary identity receipts. It contains no product archives or executables.

A fresh independent reviewer accepted the hosted evidence after checking all 132 manifest entries, all 79 raw log members, all six downloaded artifact ZIPs against API sizes and digests, every artifact member, 15 native executables, two container executables, and the platform reports against the raw logs. The review confirmed offline source isolation, both Homebrew installs, schema comparison, staging and the skipped release operator. The build delegate reverified the durable archive and removed its owned temporary downloads after acceptance. GitHub artifacts remain available. Publication and the history append remain deferred for Ian's approval.
