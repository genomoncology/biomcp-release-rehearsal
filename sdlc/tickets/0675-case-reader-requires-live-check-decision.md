# 0675 — Every case reader requires the live-check decision

Status: COMPLETE.
Landed: 95d4c674da04ac3543587adb35705aa9128fa6d3

Milestone: 5

## Outcome

Every way of reading a replayed case requires its live-check declaration and refuses a missing or blank one with the case's file name, not only the live reader. A new case that omits it fails the first routine run that reads it, in single, native and typed builds.

## Why

Twelve cases across two tickets omitted the declaration (coordinator RCA, m27067). One reached an installed hosted run and failed there (0671). The shared `Case` type makes `_live_check` optional (`case_data.rs:12-13`); `read_case` only deserializes (`:79-81`); ordinary discovery accepts it (`mod.rs:158-177`); only `live.rs:50-64` rejects it, and native builds omit the live module (`mod.rs:73`). This is the release-finds-no-bugs rule applied to fixtures (`decisions/2026-11-05-release-finds-no-bugs.md`).

## Proof

- Proof: move the existing nonblank validation into `read_case`, keeping its file name and error text, and remove the live-only duplicate. Extend the existing missing-marker case in `live_follow_proof.rs:93-118` to ordinary `--cases --list` and to a blank value; it fails on current main. Give the six old fixtures and the eight in-code `Case` constructors accurate exclusions so the negative tests keep their meaning. Focused offline proof, then the routine gate.

## Starts from

- Starts from: the files and lines above, and the coordinator's inventory in m27067 (five tool-case trial and name-fallback variants, drug label discovery, native case inputs for disease, and eight constructors in seven Rust files).

## Keeps

- Keeps: the declaration's format and meaning, the live reader's behavior, and every existing case's result. No new checker, job, default or schema change. The reader can require an explicit decision; it cannot judge whether the reason given is honest.

## Defers

- Defers: nothing.

## Changes

- Changes: the shared case reader and its tests, the six fixtures and the in-code constructors; about sixteen test-only files.
