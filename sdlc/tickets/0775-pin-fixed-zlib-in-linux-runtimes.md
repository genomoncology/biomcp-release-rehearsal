# 0775 — Pin fixed zlib in Linux runtimes

Status: COMPLETE.
Landed: de589016a950d6822220783e1b37675c74a3a51d

Milestone: 5

## Outcome

Both shipped Linux runtime container architectures use the demonstrably fixed zlib package, pinned once in the shared release manifest.

## Why

The final candidate runtime contains vulnerable zlib1g 1:1.2.13.dfsg-1. Debian identifies the trixie-security update as fixed for CVE-2026-76844 and CVE-2026-85091; bookworm has no fixed package.

Sources: https://security-tracker.debian.org/tracker/CVE-2026-76844 and https://security-tracker.debian.org/tracker/CVE-2026-85091.

## Proof

- Proof: both actual architecture images report the pinned package version and verified archive digest and pass existing runtime behavior checks. Missing, old or wrong-digest packages fail. The dated source metadata verifies under Debian signatures.

## Starts from

- Starts from: release/toolchain.json, release/Dockerfile, existing Linux producers, source policy and container check.

## Keeps

- Keeps: native Rust binary, APIs, cache behavior, bookworm base, compiler, OpenSSL, rootless runtime, provenance and signature checks. No suite upgrade, live archive, moving tag, advisory exception or fresh datasets.

## Defers

- Defers: hosting consumes the same shared pin in its ticket 0085; final candidate identity qualification follows the landing.

## Changes

- Changes: one exact package/version/source and architecture digest manifest pin, its existing runtime producers and purposeful regression tests.
