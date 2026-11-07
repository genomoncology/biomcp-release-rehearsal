# Run release sections before the cut

After the required product changes have landed, select one immutable main commit and run the sections below through `pm job` on their actual hosts. Keep each section's command, host, source commit, inputs, exit and output location in its job record. Fix a failed section and rerun that section. A product change selects a new main commit; a tooling change names its actual revision and repeats only the affected section. The candidate cut cites the completed main preflight.

This document prepares the commands. It does not establish a passing preflight. Do not start product runs while the required product work is still landing.

## Select sources and inputs

`MAIN_REVISION` is the full main commit selected after the required changes land. It is the product source, runner source and fixture source for this preflight. `DRIVER_REVISION` is the actual commit of an outer workflow or wrapper if it differs. Record both. `BIOMCP_BUILD_REVISION`, when needed by a Git archive, must identify the source actually being compiled.

Run commands from a clean checkout of selected main using its `release/toolchain.json` pins. After verifying that checkout's HEAD, set `BIOMCP_BUILD_REVISION="$MAIN_REVISION"` for producer and runner compilation, including `release/gate.py`; generated artifact directories must not cause dirty-tree identity inference. This names the source actually compiled, not a different driver tree. The artifact's `release.json`, inspected wheel and `version --verbose` must agree on the product commit and binary hash. A prepared runner's `bootstrap.json` names the source actually used to compile its case and tool runners.

The installed runner expects its own compiled commit in the product's verbose version (`crates/biomcp/tests/runner/housekeeping.rs`) and uses its linked schema types. The standalone installed-smoke command also requires product and runner source to equal the selected candidate. Therefore use product, runners and fixtures from the same `MAIN_REVISION`. A newer runner checking an older executable is not proof of new main. Do not spoof stamps, skip housekeeping or weaken schema checks to make that pairing pass.

Artifacts from an earlier main commit do not qualify new main. A section that needs hosted artifacts takes the newest retained ones and names their product commit. Keep earlier failures and passes under their original identities.

| Input | Contents |
| --- | --- |
| `NATIVE` | One platform's complete producer output: wheel, `release.json`, archive and checksum, registry record, licenses and desktop bundle where produced |
| `PROOF` | Matching case runner, tool runner, copied fixtures, copied release helpers and `bootstrap.json`, produced by `release/gate.py --prepare-only` |
| `ALL_NATIVE` | One subdirectory per native platform, each containing producer output from the same selected main commit |
| `STAGED` | A new output directory for the complete publication payload; absent before staging |
| `SCRATCH` | A fresh installation directory owned by one gate invocation |
| `EVIDENCE` | Retained outputs, reports and failed synthetic fixtures; never a home directory or the whole operating-system temporary directory |
| `VERSION` | The selected product's Cargo version, also present in its producer records |
| `BINARY` | An installed or unpacked executable from `NATIVE`, whose hash the channel helper checks against the wheel |
| `PLUGIN_SOURCE` | The unpacked `biomcp-plugin.zip` from `STAGED` |

Copy outputs needed by another job into the owning team's retained evidence store before producer cleanup. Record their product revision, runner revision, hashes and producer job. Do not retain credentials, Git authentication files or unrelated runner temporary files.

## Order and commands

Independent rows in one phase may run in parallel when outputs do not collide. Run them through tracked pm jobs. Name a machine with `--machine` only when the section needs its operating system or data held on that host. For example:

```sh
pm job run 0640 "$MAIN_REVISION" -- make full-suite
```

Paths and environment inputs must exist on that job's host. A missing artifact, executable, host or tool is a not-run or failed section, never a pass. These are existing commands; the local sections require no new launcher or framework.

### 1. Source checks and mutations

| Section | Command | Host and inputs |
| --- | --- | --- |
| Full suite | `make full-suite` | Any Linux or macOS host; complete selected-main history, not a shallow archive; manifest-pinned `cargo-deny` and `actionlint` installed first |
| Release schema comparison | `make schema-release-check` | Any Linux or macOS host; selected main and its committed schema baseline/reviews; does not update the baseline |
| Mutation tool, if the pin is absent | `make release-mutations MUTATION_REPORT="$EVIDENCE/mutation-install.json" MUTATION_ARGS="install --started $STARTED --output $MUTATION_TOOL"` | Any Linux or macOS host; a new owned tool directory and the existing shared 90-minute budget |
| Mutation plan | `make release-mutations MUTATION_REPORT="$EVIDENCE/mutation-plan.json" MUTATION_ARGS="plan --previous $PREVIOUS --started $STARTED --output $MUTATIONS --cost-seconds $COST --baseline-seconds $BASELINE --shards 1"` | Clean selected main and previous compared commit; measured cost from retained native calibration |
| Mutation shard | `make release-mutations MUTATION_REPORT="$EVIDENCE/mutation-shard.json" MUTATION_ARGS="shard --plan $MUTATIONS/plan.json --index 0"` | The plan's native host, selected main and retained plan/inventory |
| Mutation collection | `make release-mutations MUTATION_REPORT="$EVIDENCE/mutation-collect.json" MUTATION_ARGS="collect --plan $MUTATIONS/plan.json"` | The same plan and actual shard outputs; retain survivors, timeouts and unrun work alongside the exit |

`make full-suite` stops at its first failed stage. Its stages are `audit`, `secrets-full`, `pre-job`, `workflow-lint`, `docs-check`, `tensor-proto-check`, `library-tests` and `coverage`, in that order. Each is a Make goal, so after a fix rerun only the failed stage with `make STAGE`. `make coverage` also repeats `docs-check` and `library-tests` unless each is passed with `-o`, as the suite does. The coverage stage reports coverage; no floor gates it.

`install` puts the tool in `$MUTATION_TOOL/tool/bin`. Put that directory first on `PATH` for the later actions, which refuse to plan without the pinned tool.

`STARTED` is one Unix start time shared by the actions of one run, not reset for a shard retry. With no retained measurement, first run the plan, shard and collection actions with `--measure` in place of `--cost-seconds` and `--baseline-seconds`, into their own output directory and with their own `STARTED`. That selects the existing 20-mutant sample and only measures cost, so its survivors do not fail it and it does not pass this section. `COST` and `BASELINE` are `seconds_per_viable_mutant` and `baseline_seconds` from that run's `mutation-report.json`. Use them only when its `calibration_complete` is true, because a calibration's collection exits zero even when it was cut short.

Collection's exit is this section's result. A shard exits zero on survivors, on timeouts and when the deadline has already passed and nothing ran. A planned run fails collection on any survivor, timeout or selected mutant left unrun. Collection checks only the selected mutants, which the time budget can cut to a small part of the scope, and it exits zero when every selected mutant is unviable. A pass therefore also needs caught mutants in `mutation-report.json`, whose `unrun` list shows what was never tried. Retain survivors, timeouts and unrun work alongside the exit. To retry a shard, first move its `shard-N` directory aside, because the action refuses one that exists.

### 2. Native production, installed gates and cold Linux

| Platform | Real host | Existing production command |
| --- | --- | --- |
| Linux x86_64 | Native Linux x86_64, including `ubuntu-24.04` | `make release-linux`, with `BUILD_IMAGE` from manifest key `manylinux-x86_64` |
| Linux arm64 | Native Linux arm64, including `ubuntu-24.04-arm` and Linux arm64 containers on an arm64 host | `make release-linux`, with `BUILD_IMAGE` from manifest key `manylinux-aarch64` |
| macOS arm64 | Apple Silicon macOS, including `macos-15` | `python3 release/artifacts.py build --revision "$MAIN_REVISION" --output "$NATIVE"` |
| Intel macOS | `macos-15-intel` | The same native production command, executed on Intel macOS |
| Windows x86_64 | `windows-2022` | `python release/artifacts.py build --revision "$MAIN_REVISION" --output "$NATIVE"` |

`release/linux.sh` already builds the manylinux artifacts, prepares matching runners, runs installed forms without the checkout and checks the matching runtime container. Retain its actual install report, raw log and container receipt. Do not rebuild that platform merely to repeat those checks.

A release-profile build refuses a binary that still names a folder of the machine that built it: the checkout's `crates` folder, Cargo's target folder, the vendored source folder or the Cargo registry. That refusal fails this section on any platform. Until a preflight passes, this check has real proof only on macOS arm64, so every other platform proves it here.

The runtime container installs its packages only from the dated Debian snapshot named by `debian-snapshot` in `release/toolchain.json`, over plain http, with the signed Release file carrying the trust. A snapshot that cannot be reached fails the container build; do not switch to the live archive. Until a preflight passes, a real build from the snapshot has proof only on arm64 and, emulated, on amd64, so native amd64 proves it here.

| Section | Command | Host and inputs |
| --- | --- | --- |
| Native installed forms | `python3 release/gate.py --artifacts "$NATIVE" --scratch "$SCRATCH" --python python3` | Matching native host; product and runner source both selected main; use `python` on Windows and a writable job-owned `BIOMCP_EVIDENCE_DIR` |
| Prepare retained runners without rebuilding product | `python3 release/gate.py --prepare-only --artifacts "$NATIVE" --scratch "$PROOF"` | Matching source/platform; reuse an existing prepared proof when available |
| Prepared installed gate | `python3 release/gate.py --prepared --artifacts "$NATIVE" --scratch "$SCRATCH" --report "$EVIDENCE/install-report.json" --python python3` | Matching platform; first copy only the prepared proof into fresh `SCRATCH`; use Windows Python there |
| Linux runtime container alone | `python3 release/container_check.py "$IMAGE" "$ARCH" "$CONTAINER_PROOF" "$VERSION"` | Matching Linux architecture and retained same-main image; proof directory contains binary, licenses, runners and fixtures; `ARCH` is `amd64` or `arm64` |
| Cold Linux matrix | `make release-cold COLD_REPORT="$EVIDENCE/cold-report.json"` | A host with Docker and both pinned images; `snapshot.debian.org` reachable for the `debian-snapshot` date in `release/toolchain.json`; empty container homes; existing command collects every failed stage across both architectures |
| Cold Linux architecture retry | `make release-linux CONTAINER_ARGS="--platform linux/$ARCH"` | Selected main with `BIOMCP_RELEASE_COLD=1`, `BIOMCP_BUILD_REVISION="$MAIN_REVISION"`, corresponding pinned `BUILD_IMAGE` and owned `BIOMCP_RELEASE_REPORT`; `snapshot.debian.org` reachable for the `debian-snapshot` date in `release/toolchain.json`; only that architecture's existing cold recipe |

On macOS, a gate without Homebrew exits zero and marks `homebrew-formula` as `deferred` in its report. That section did not run: require `pass` there, as the hosted jobs do.

Label x86_64 container runs on an arm64 host as emulated. They do not replace the native Linux x86_64 result. The cold matrix retains product outputs in `cold-artifacts-amd64` and `cold-artifacts-arm64`, then removes its temporary prepared proofs. Final hosted native ZIPs also do not currently retain runner executables. If a later consumer needs `PROOF`, prepare it once from the same main source with the existing command above, then retain and reuse it.

The recovery action builds and qualifies one platform alone at a main revision. It is the route to Intel macOS and Windows, which have no local release host. Each dispatch is manual, like the rehearsal in section 5, and selected main must already be in the rehearsal repository:

```sh
gh workflow run recover-section.yml --repo genomoncology/biomcp-release-rehearsal --ref "$RECOVERY_REF" -f section="$SECTION" -f main_revision="$MAIN_REVISION" -f main_tree="$(git rev-parse "$MAIN_REVISION^{tree}")" -f rehearsal=true
```

`SECTION` is `manylinux_2_28_x86_64`, `manylinux_2_28_aarch64`, `macosx_11_0_arm64`, `macosx_11_0_x86_64` or `win_amd64`. `RECOVERY_REF` is a `rehearsal-0642-*` branch of the rehearsal repository, and its commit is the driver revision. Product, runners and fixtures come from the selected main checkout. `release/section.py` refuses a commit or tree that differs from the request, and a driver whose `Cargo.lock`, `rust-toolchain.toml` or `release/toolchain.json` differs from main's. The action publishes nothing. Its `recovered-$SECTION` artifact holds the producer output, install report and source record; retain it and use its `dist` as that platform's `NATIVE`.

### 3. Installed smoke and local channels

| Section | Command | Host and inputs |
| --- | --- | --- |
| Full isolated installed cleanup smoke | `python3 scripts/installed_smoke_section.py "$NATIVE" "$PROOF" "$EVIDENCE/installed-smoke" --candidate "$MAIN_REVISION"` | Native Linux x86_64; retained matching product, runners and fixtures; new output directory; no compilation in this section |
| Pinned public 0.9 installer | `python3 release/published.py legacy-installer --version "$VERSION" --revision "$MAIN_REVISION" --artifacts "$NATIVE"` | Native Unix host matching the archive, with curl; optional `--installer-script` supplies the exact pinned cached public script |

The legacy helper verifies the pinned 0.9.3 installer bytes and redirects only its exact archive and checksum requests to the existing loopback server. It publishes nothing and uses no source API.

Windows release proof is the hosted native `release/gate.py` result from the recovery action or the rehearsal, as [the release decision](../sdlc/decisions/2026-11-05-release-finds-no-bugs.md) records. The VM wrapper remains outside the release path after its missing-file fault. After machine diagnosis, its existing command is `sh scripts/check_windows_release.sh 0640 "$ARTIFACT_REVISION" "$NATIVE"` on the Windows VM host, under an owning pm job with the same ticket label. It must report actual artifact and tooling revisions. An old artifact can exercise that wrapper but cannot qualify new main.

### 4. Complete payload preparation and screening

| Section | Command | Host and inputs |
| --- | --- | --- |
| Publication payload | `python3 release/workflow.py stage --artifacts "$ALL_NATIVE" --plugin . --version "$VERSION" --revision "$MAIN_REVISION" --output "$STAGED"` | Any Linux or macOS host; all five same-main native outputs, both Linux container archives and selected-main plugin source |
| Staged artifact secret scan | `python3 release/secrets.py artifacts "$STAGED"` | Actual complete staged bytes and pinned scanner; includes archive members and compiled binaries. An empty directory scans clean, so this row counts only with a passing subject inventory of the same `STAGED` |
| Staged Homebrew formula | `python3 release/gate.py --artifacts "$NATIVE" --scratch "$SCRATCH" --staged-formula "$STAGED/biomcp.rb"` | Apple Silicon macOS with its matching native output and the universal staged formula. Intel has no local host and no recovery section, so its half runs only as the rehearsal's `staged-homebrew` job on `macos-15-intel` |
| Claude plugin | `python3 release/published.py plugin --version "$VERSION" --revision "$MAIN_REVISION" --artifacts "$NATIVE" --plugin-source "$PLUGIN_SOURCE" --binary "$BINARY" --claude "$CLAUDE"` | Native Unix host with real Claude CLI; same-main staged plugin and executable; no model invocation |
| Attestation subject inventory | `python3 release/workflow.py subjects --artifacts "$STAGED" --output "$EVIDENCE/subjects.sha256"` | Complete staged tree; new output outside that tree |

Staging does not scan the payload by itself. Screen those bytes before treating stage preparation as complete. History screening does not replace artifact screening. Classify a failure and fix its owning source; do not add a broad binary or archive allowlist. Each staged Homebrew run installs the actual two-architecture formula from the checked local archives, without fetching a public release. Unpack the checked stage's plugin archive for the plugin row. Its helper installs the marketplace in an isolated home, reads the installed launch declaration and calls the selected executable's MCP `help`, without invoking a model or a source API.

### 5. Ordinary hosted rehearsal and signed artifacts

After the independent sections pass, run the existing normal release workflow once from selected main in the approved public rehearsal repository, with `rehearsal=true` under an approved non-release ref, as [the public rehearsal decision](../sdlc/decisions/2026-11-02-public-rehearsals.md) describes. Do not replace the normal workflow with a recovery workflow for this check. This document does not dispatch it or approve a release tag.

The rehearsal runs the staged sections again in `release.yml`, on the bytes the workflow built:

| Section | Job and host | Alone |
| --- | --- | --- |
| Stage | `stage`, `ubuntu-24.04` | The publication payload command in section 4 |
| Staged artifact scan | `stage`, `ubuntu-24.04` | The staged artifact secret scan command in section 4 |
| Attestation | `stage`, `ubuntu-24.04`, from the subject inventory | Signing needs the workflow's identity, so it has no local command and reruns with this rehearsal. The inventory runs alone in section 4 and `gh attestation verify` below checks each file |
| Staged Homebrew | `staged-homebrew`, `macos-15` and `macos-15-intel` | The staged Homebrew formula command in section 4 on Apple Silicon. Intel reruns with this rehearsal |

```sh
gh workflow run release.yml --repo genomoncology/biomcp-release-rehearsal --ref "$REHEARSAL_REF" -f rehearsal=true
```

Bind the actual run commit, tree and normal workflow bytes to selected main. Product and runner stamps name the source actually compiled. Retain actual job results for candidate policy/history, all five native platforms, both Linux containers, staging and artifact screening, signed subjects, both staged Homebrew architectures, and downstream rehearsal jobs. A successful older product is not evidence for new main.

For each downloaded staged subject, first compare the file with the retained subject inventory, then verify its signed provenance with a current GitHub CLI:

```sh
gh attestation verify "$ARTIFACT" --repo genomoncology/biomcp-release-rehearsal --signer-workflow genomoncology/biomcp-release-rehearsal/.github/workflows/release.yml --source-ref "$REHEARSAL_SOURCE_REF" --source-digest "$MAIN_REVISION"
```

Use the actual full source ref and require the certified run to match the retained run. Every staged regular file must appear exactly once in the subject inventory. Checksums alone do not establish signed provenance. If screening stops staging before signing, subjects and downstream jobs are not run; native successes alone do not make the rehearsal pass.

The six downstream channel jobs cover GitHub assets, GHCR, PyPI, Homebrew, the installer site and MCP Registry. Dispatch skips their authentication and publication steps. The five post-publication install jobs are also no-ops, and latest-pointer promotion stays unchanged. Record these boundaries: their summaries do not prove real public-channel installation, registry acceptance or promotion. Publication still needs Ian's approval.

## Tooling checks do not qualify product

`sh scripts/check_release_channels.sh` checks channel helpers with small test candidates. `sh scripts/check_hosted_section.sh` checks recovery tooling. `python3 scripts/installed_smoke_section.py check` checks the standalone smoke helper. Run those existing focused checks when their tooling changes; they do not replace the real-host sections above. `make landing-gate` is the short check every landing runs; it qualifies no section.
