# 0730 — Build current-main shakedown binaries for agent evaluation

Status: OPEN.

Milestone: 5

## Outcome

Build one Linux x86-64 and one macOS arm64 shakedown executable from the same pinned current-main commit. Name their retained locations by that commit, make them readable by the agent bench, and report the source commit, paths and SHA-256 checksums to the PM.

## Why

Ian prioritized the agent-flavor evaluation; PM message m28928 requests this pair now without displacing the typed batch.

## Proof

- Proof: Retain each build job's exact source binding, executable version and native architecture, and verify each receiver's executable checksum. Report both paths and hashes together.

## Starts from

- Starts from: Current `origin/main`, existing tracked build goals and the bench's existing artifact input mechanism.

## Keeps

- Keeps: One identical source commit for both executables. Typed-batch work keeps priority. Jobs use automatic placement except the macOS build explicitly uses M5. No product changes or publication.

## Defers

- Defers: The scored build waits for the typed, text-rights and life-sciences fixes named in PM message m28928 to land.

## Changes

- Changes: `sdlc/tickets/0730-bench-shakedown-binaries.md` and retained build outputs.
