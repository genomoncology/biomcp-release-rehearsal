# 0533 — Publish the release through gated jobs and check every channel

Status: COMPLETE.
Landed: c4eedced3301cae52c02ae7efc3b84e4ac80c736

Milestone: 5

## Outcome

The release workflow publishes to every channel through gated jobs in the `release-publication` environment, attests the pushed container digest, runs the docs check on the tag, and installs from each channel after publishing. The public repository settings and credentials Ian must provide are listed in one place.

- Why: release review C1, B3 and C11, weighed 2026-11-01. `.github/workflows/release.yml:231-253` prints that it grants no credentials and performs no publication, and no ticket names who publishes, with what credentials, or what is checked afterwards. `release.yml:165-172` attests file hashes with `push-to-registry: false`, so pulled images cannot be verified, and PyPI uploads use neither Trusted Publishing nor attestations. `release.yml:113` runs `make fmt lint test` on the tag, without the docs check.
- Starts from: `release.yml`, `release/workflow.py`, `release/artifacts.py`, the 0.9 release workflow in the published repository's history, and 0294.
- Rules: GitHub Actions runs only public releases. Each publish job needs the environment's approval. PyPI uses Trusted Publishing; the container job attests the pushed digest. A post-publish job installs from PyPI, GHCR, Homebrew, the installer script and the MCP registry, and runs `biomcp version`. Creating the PyPI trusted publisher, the environment reviewers and any tokens are external changes Ian approves; the ticket lists them and does not perform them. Add `docs-check` to the tag job.
- Proof: actionlint passes; a dry run on a fork or a scratch tag without credentials reaches each gated job and stops at approval; the settings list is complete against the jobs.
- Changes: the release workflow, release scripts and a short operator section in `docs/development.md` without private names.
- Defers: nothing.
