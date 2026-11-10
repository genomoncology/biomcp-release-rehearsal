# 0806 — Match the existing PyPI Trusted Publisher environment

Status: COMPLETE.
Landed: a229f75226429a5b5988ebf39a492db2b30936ae

Milestone: 5

## Outcome

Only the PyPI publishing job uses GitHub environment pypi, matching the existing biomcp-cli Trusted Publisher. Other publication/promotion jobs retain release-publication. PyPI retains its dependency on gated GHCR publication, production tag/repository guards and OIDC attestations.

## Why

Ian supplied the actual publisher settings in chat: genomoncology/biomcp, release.yml, environment pypi. The workflow currently uses release-publication for that job, an identity mismatch that would prevent Trusted Publishing. Read-only GitHub API inspection confirms both environments already exist, so fix code rather than change third-party settings.

## Proof

- Update existing publication-contract checks to require pypi for publish-pypi and release-publication elsewhere, preserve publish-ghcr dependency and exact production event/repository guards, and retain rehearsal no-auth/no-publication behavior.
- Demonstrate the corrected environment contract fails on the existing workflow and passes after the narrow fix. Run existing focused publication/promotion/workflow checks and routine gate; no PyPI authentication, upload or account mutation during testing.
- Verify documentation gives the existing correct Trusted Publisher fields and does not require changing account configuration or adding manual approval pauses.

## Starts from

- Selected rehearsal source ca945a56ac403d0c823c6b701bc813a9270da8da, .github/workflows/release.yml publish-pypi, release/test_publication.py and docs/development.md. Existing publish-pypi needs publish-ghcr, which retains release-publication. GitHub pypi exists without protection rules; release-publication retains configured tag deployment policies. No configuration writes are authorized by this ticket.
- PyPI's primary documentation describes matching owner, repository, workflow filename and optional environment: https://docs.pypi.org/trusted-publishers/adding-a-publisher/ . The human-confirmed environment is pypi, not release-publication.

## Keeps

- Every automatic qualification, staging, channel ordering, credential restriction, attestation, post-publication install and promotion condition. Ian approves the real tag before any publication; there is no added workflow pause. Runtime, schemas, artifacts, dependency pins and installed tests are unchanged.
- Preserve the running ca945a56 rehearsal and its actual source identities. This workflow-only fix changes publishing configuration, not the runtime-platform inputs; record actual ending tooling revision and assess only the affected workflow proof without relabelling earlier artifacts or blindly repeating native builds.

## Defers

- Real PyPI publication remains part of the separately approved release session. This fix cannot prove a live OIDC exchange through a rehearsal that intentionally skips authentication; retain that boundary.

## Changes

- PyPI job environment, existing publication-contract checks, and directly contradictory operator documentation. No GitHub/PyPI account settings or new release framework.
