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

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

`profile.trim-paths` is refused by cargo 1.95.0. Use `--remap-path-prefix` in the temporary `config.toml` that `release/vendor.py` writes, as `[target.'cfg(all())'] rustflags`, where the vendor path is known.

Do not use `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS`. Either replaces the target `rustflags` in `.cargo/config.toml`, and the Windows build would lose its static C runtime with no error.

The check belongs in `release/artifacts.py build()` so every platform runs it. It searches for the source `crates` path, the vendor folder and `CARGO_HOME`. In containers the root is `/source`, so a search for the bare root gives false matches. The Proof runs through `release/artifacts.py build`, not plain `cargo build --release`. The section preflight in 0640 is the Windows and Intel proof.
