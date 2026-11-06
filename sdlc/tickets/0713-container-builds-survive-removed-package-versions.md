# 0713 — Container builds survive removed package versions

Status: OPEN.

Milestone: 5

## Outcome

The release container still builds when Debian retires a pinned package version, and it installs the same pinned versions.

## Why

Row W54 of `records/2026-11-06-pre-release-review.md`, first rejected because it fails safely; Ian asked for these to be done on 2026-11-06. `release/toolchain.json` pins exact versions that the normal archive drops.

## Proof

- Proof: a build against the dated Debian snapshot in `release/toolchain.json`, on a host that can run containers. A security update still needs one deliberate date change.

## Starts from

- Starts from: `release/Dockerfile`, `release/toolchain.json`.

## Keeps

- Keeps: exact version pins and the container digest checks.

## Defers

- Defers: nothing.

## Changes

- Changes: `release/`.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

All four pins are in the normal Debian archive today, so a build against it proves nothing about removed versions. The Proof changes: the build's package sources point only at the dated snapshot, and the build log shows every package fetched from it.

`release/toolchain.json` holds no snapshot date yet. Add a `debian-snapshot` key next to the pins, pass it as a build argument from `release/linux.sh` and `release/matrix.sh`, and extend `release/toolchain.py` and its tests.

Use plain `http://snapshot.debian.org/archive/debian/<date>/` for bookworm main only, with `Acquire::Check-Valid-Until=false`. The slim image has no CA certificates before this step, and the signed Release file provides trust. There is no fallback to the live archive: if the snapshot is unavailable, the build fails.
