# 0712 — Release binaries carry no build paths

Status: OPEN.

Milestone: 5

## Outcome

A release binary holds no folder path from the machine that built it.

## Why

Row W42 of `records/2026-11-06-pre-release-review.md`, first rejected as hygiene; Ian asked for cosmetic issues to be done on 2026-11-06. No private names were found, so this is cleanliness, not a leak.

## Proof

- Proof: a release-profile build on one host, failing first, whose binary is searched for the build folder and home folder and finds neither. The check runs with the release tooling checks, not only at release.

## Starts from

- Starts from: `.cargo/`, `release/`, and the build steps in `.github/workflows/release.yml`.

## Keeps

- Keeps: panic locations usable for bug reports.

## Defers

- Defers: nothing.

## Changes

- Changes: build settings and release tooling.
