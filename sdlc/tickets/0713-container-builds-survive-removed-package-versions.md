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
