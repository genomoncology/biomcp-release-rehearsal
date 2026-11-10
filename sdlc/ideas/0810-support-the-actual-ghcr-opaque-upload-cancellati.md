# Support the actual GHCR opaque upload cancellation contract before reusing the credential probe

Status: open.

Kind: idea
When: Before any future live use or shipping of the archived 0807 probe helper; not a 1.0 runtime or credential-readiness blocker

## Problem

Support the actual GHCR opaque upload cancellation contract before reusing the credential probe. Both live probes established GHCR upload admission, but cleanup rejected the provider path and was not attempted. Two empty sessions have unconfirmed cleanup; no payload, manifest or tag was sent. Preserve failed runs 42884420606 and 42885938241, actual source, safe diagnostics and no raw URL/token disclosure. Resolve the documented opaque Location contract and review destination/cleanup safety before any new live session; do not broaden guards or retry blindly.
