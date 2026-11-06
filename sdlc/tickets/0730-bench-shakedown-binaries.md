# 0730 — Build current-main shakedown binaries for agent evaluation

Status: COMPLETE.
Landed: 61d24fb3e361f013c7b6e3a4a9639df4a2cdbea2

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

## Retained shakedown pair

Both executables were built from pinned current main `45eac64ed1863ee2a78a40ee11026b7cb9bf1b9a` using the existing `plain-cli-build` goal, Rust 1.95.0 and the unoptimized dev profile. Linux job j32572 used automatic placement on Gator; macOS job j32573 ran explicitly on M5. Both jobs passed. These are internal shakedown inputs; scored-build and release qualification remain deferred.

Retained directory on Beelink: `/home/ian/workspace/data/biomcp-redux/internal-build/45eac64ed1863ee2a78a40ee11026b7cb9bf1b9a/`.

| Executable in that directory | SHA-256 | Native architecture and version |
| --- | --- | --- |
| `biomcp-linux-x86_64` | `2e0e808f833e9f637dd509583ea195b061af59c98a411dee4e98ba6bab8d91b7` | ELF x86-64; `biomcp 1.0.0` |
| `biomcp-macos-arm64` | `4a576fd6832f87dab5ac6b0856b74b4b5f6745d913e8a5599ac35bc7ffcb8cda` | Mach-O arm64; `biomcp 1.0.0` |

The Mac executable is also retained on M5 at `/Users/ian/workspace/data/biomcp-redux/internal-build/45eac64ed1863ee2a78a40ee11026b7cb9bf1b9a/biomcp-macos-arm64`; its receiver checksum matches the Mac producer checksum. Both Beelink files are readable and executable, and independently passed `sha256sum -c SHA256SUMS`. Independent `file` inspection confirmed both architectures, and the Linux receiver returned `biomcp 1.0.0`. The Mac version comes from its native M5 proof; it was not executed on Linux. The bench's existing `tools/live-biomcp.sh` accepts the executable path as its `BIOMCP` argument.

Evidence beside the pair: `receipt.json`, `SHA256SUMS`, `linux-source-binding.txt`, `macos-source-binding.txt`, `j32572-linux-build.log`, `j32573-macos-build.log`, and `job-results.txt`. The raw job logs bind both builds to the pinned source and dev profile; source-binding files retain toolchain, producer checksum, native architecture and version, including the Mac receiver checksum. No product source or build configuration changed.
