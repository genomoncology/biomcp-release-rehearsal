# Hosted qualification prepares writable installed checks with explicit tooling identity

Status: COMPLETE.
Landed: d4123ff8054939ee0d9949992c17c2fa53e2dee9

Milestone: 5

## Outcome

The failed hosted qualification sections run their installed checks for product candidate `1bb97e64f41634b19efc42b937b195e55a20ece4`, with a writable evidence directory and an explicit, truthful tooling revision. Each section can be recovered alone without restarting passing sections or changing product source (Ian, relayed in m26087).

## Why

Hosted run `42030695983` stopped Linux arm64 at `/target/fixture-evidence` permissions and stopped macOS arm64 and Windows at dirty-tree tooling identification, after their producers ran.

## Proof

- Proof: recover only the failed sections and retain their actual installed results, product and tooling identities, and source/artifact hashes. Reuse completed producer artifacts when retrievable; otherwise rebuild only that section's unchanged product. Existing successful sections remain valid.

## Starts from

- Starts from: `.github/workflows/`, `release/linux.sh`, `release/gate.py`, and the local native recovery's explicit revision handling. Actual failed logs are on Yellow at `data/biomcp-redux-evidence/0293/candidate-1bb97e64f416/hosted-42030695983/initial-failures/`: Linux arm64 job 127257791290 line 1965; macOS arm64 job 127257791390 line 1305; Windows job 127257791408 line 1293.

## Keeps

- Keeps: unchanged product source and artifact selection, pinned tools, full installed case/tool/TLS checks, isolated caches/settings, exact product-versus-tooling provenance, and publishing disabled. A dedicated action or existing job recovery selects the affected section, as Ian requested in m26087.

## Defers

- Defers: product changes, unrelated sections, publication and public BioMCP ref changes.

## Changes

- Changes: hosted qualification workflow and its existing release preparation helpers; tooling only.
