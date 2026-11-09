# 0293 — Rehearse the final release candidate on every platform

Status: COMPLETE.
Landed: 71d40bf94f8f9c3bcaa4c4ae0c21dd8f43ef6726

Milestone: 5

Depends on: 0289, 0290, 0292, 0297, 0319, 0332, 0336, 0621, 0622, 0634, 0635, 0636, 0638, 0639, 0640, 0651, 0652, 0653, 0654, 0655, 0656, 0657, 0658, 0659, 0660, 0661, 0662, 0663, 0665, 0666, 0667, 0668, 0669, 0671, 0675, 0677, 0678, 0679, 0680, 0681, 0682, 0683, 0684, 0686, 0687, 0641, 0649, 0670, 0676, 0689, 0690, 0691, 0692, 0693, 0694, 0695, 0696, 0697, 0710, 0711, 0712, 0713, 0714, 0715, 0717, 0718, 0719, 0720, 0721, 0722, 0723, 0724, 0725, 0726, 0727, 0729, 0731, 0732

## Outcome

The final release candidate's artifacts install and run on every promised platform, repeating the 0066 installed rehearsal: wheel, installer archives, Homebrew formula, container, desktop bundle and registry entry, each running version, health, replayed search and get, and stdio and HTTP MCP. The report names any platform that could not be exercised.

- Why: the release plan requires final-candidate installed proof before publication approval.
- Starts from: `release/qualification.md` and the ordinary release workflow.
- Rules: run the approved public rehearsal with `rehearsal=true`; retain genuine product, runner and workflow identities. Publishing remains separate. `0716` was withdrawn to keep both wheel launch binaries for 1.0; removing that stale dependency changes no packaging.
- Proof: the final run, platform reports, staged subjects and independent provenance verification are reviewed against the selected source before this ticket closes.
- Changes: this evidence record only.

## Final rehearsal evidence

[Run 42680531044](https://github.com/genomoncology/biomcp-release-rehearsal/actions/runs/42680531044) completed successfully with all 22 jobs. It ran the ordinary `release.yml` via manual dispatch, `rehearsal=true`, from `refs/heads/rehearsal-0293-b3809eb1baa6`.

Product and runner source: `b3809eb1baa6d1e470efaf6499d8be0ba600f511`. Tree: `4baf7bddd9487d5adba82589c45351bef36fa2b2`. Workflow blob: `c0d772aeffd6f921cf76b2ba3e6e470976f45cbf`. The product stays fixed through later evidence-only commits.

All five native platforms passed. Each macOS architecture installed six channels with 740 cases per channel, including real Homebrew. Each Linux architecture installed five channels with 740 cases per channel; Windows installed five with 734. Both native Linux runtime containers passed. Linux and Windows reports explicitly defer Homebrew.

The complete staged payload, artifact secret scan, both staged Homebrew architectures, strict wheel metadata and staged installer-byte comparison passed. Every one of the 26 staged regular files occurs once in the checksum inventory, matches its retained bytes and independently verifies against the signed provenance using GitHub CLI 2.92. Verification required the exact repository, signer workflow, source ref and source digest; each verified statement identifies this run.

The retained, receiver-verified packet is Yellow `data/biomcp-redux-evidence/0293/b3809eb1baa6/run-42680531044/`:

- `run-terminal.json`, `jobs-terminal.json`, `artifacts-terminal.json` and `raw-terminal-logs.zip` retain the actual run.
- `download-receipt.json` binds all nine downloaded native/staged/report artifact ZIPs to their API IDs, byte counts and SHA-256 digests, and lists skipped production steps.
- `native-reports-verified.json` binds each native wheel and staged wheel to its release record and installed results.
- `subject-inventory-verified.json`, `provenance-bundles.jsonl`, `provenance-verified/` and `provenance-verification-receipt.json` retain all 26 actual signature checks.
- `receiver-manifest.json`, `provenance-helper-receiver-manifest.json`, `cleanup-helper-receiver-manifest.json` and `docs-check-receiver-manifest.json` record receiver hash verification. Local-channel helper evidence is shared with 0640.

The workflow uploads `dist`; it does not archive every test-runner executable. The logs and native bootstrap records retain actual runner source identities, and the installed housekeeping checks require the executable's source to match its runner. The local installed cleanup helper additionally records hashes of its actual retained native macOS runners. No native runner-binary archival claim is made for the hosted jobs.

The previous `42a79b1bbdc9` run remains an explicitly earlier full-pipeline proof under its own packet. It does not provide the selected product's artifact identity.

## Acceptance boundary

All production authentication, uploads, public-channel install checks and promotion steps were skipped. Six channel jobs, five post-publication jobs and the promotion job reached their no-op rehearsal boundaries; their success does not prove public service acceptance. Nothing was published and no latest pointer moved. The preflight ticket retains the separate local sections. Its pinned old installer and real Claude plugin checks passed in `j36486`; native macOS installed cleanup passed in `j36488`; and documentation validation passed in `j36499`. Fresh evidence review remains required before closure.

## Publish commands unproved until the tag

From 0697. The rehearsal skips every step below, so each first runs at `v1.0.0`. Before the tag, the `staged-payload` job checks the staged wheels with `twine check` and compares the staged `install.sh`, and `staged-homebrew` installs the staged formula. None of these contacts a channel. No TestPyPI or scratch registry stands in.

| Job | Command that stays unproved |
| --- | --- |
| `publish-github` | `release/publication.py github`: `gh release create` |
| `publish-ghcr` | GHCR login, `release/publication.py ghcr`: `docker push` and `docker manifest`, then the container attestation |
| `publish-pypi` | `release/publication.py wheels`, then `pypa/gh-action-pypi-publish` with Trusted Publishing and attestations |
| `publish-homebrew` | The tap checkout with `HOMEBREW_TAP_TOKEN`, then `release/publication.py homebrew`: the tap commit and push |
| `publish-installer` | The `gh-pages` checkout, then `release/publication.py installer`: the commit and push, and the Pages build request |
| `publish-registry` | The cosign install, `release/toolchain.py install-mcp-publisher`, then `release/publication.py registry`: `mcp-publisher login github-oidc` and `publish` |
| `post-publish` | `release/published.py` for `pypi`, `ghcr`, `homebrew`, `installer` and `registry` against the public channels |
| `publish-latest` | GHCR login, `release/publication.py promote`: the latest image and `gh release edit --latest` |
