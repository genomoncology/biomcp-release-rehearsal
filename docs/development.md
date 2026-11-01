# Development

Run `make schema-generate` after changing schema types. The command uses the schema contract test to check compatibility before writing the checked-in schemas. It fails unless the named generation test runs and passes.

Run `make schema-release-check` to compare the generated schemas with the recorded release baseline and check for drift. This command leaves the checked-in schemas and release baseline unchanged.

## Compare replies while restructuring

Set `BIOMCP_TRANSCRIPT_DIR` to an empty output directory when running the case runner. It records exact stdout bytes, exit codes, surfaces and arguments before assertions or JSON parsing. Without the variable, tests run normally. Keep transcript directories outside Git.

Run the same selected cases and trials at the base and head commits. Then run `python3 scripts/reply_diff.py BASE HEAD`. A difference exits nonzero and names the step and JSON pointer, or a character offset for text and formatting differences. The default report stops after twenty differences. This comparison is optional and does not run in the routine gate.

Case keys contain the relative case path and zero-based step index. Each case transcript holds its CLI and MCP executions in order. Walk pages append to that same step. MCP records the raw server stdout before parsing. Ordinary trial keys contain the trial name, an argument hash and the number of earlier identical argument lists. The launcher allocates those keys before execution. Isolated environment and checkout paths become placeholders for hashing and comparison.

The comparison normalizes build identity and declared runner paths in `scripts/reply_diff.py`. `scripts/reply_runtime.py` binds supported runtime identifiers, clocks and owned paths within their captured context and checks that their references remain consistent. Its code and calibration tests own the exact substitutions. Public source values remain significant. A new substitution needs a demonstrated runtime-only difference and a calibration test.

The comparator does not support every native exchange, HTTP or JSONL form, owned path, clock or pool representation. Unsupported forms can return differences or a tool error. Keep that result and inspect the complete records field by field. Account for all public values, ordering, omissions, errors, status, provenance and follow-up commands. Preserve the original reports and identify every accepted runtime difference and every unresolved difference. Do not mask an unknown value or describe a failed comparison as green. A zero exit does not prove that every operation was captured.

Capture covers YAML steps and trial commands whose existing consumer obtains complete stdout and status. It leaves process transport and lifecycle checks unchanged. The script lists every trial with uncaptured commands or no captured command. A structure change touching these areas must use those trials' own assertions. The architecture decision keeps this tool in its optional diagnostic role. It does not require extending capture coverage.

## Exercise installed artifacts

The release rehearsal copies all of `tests/` outside the checkout and runs the complete `cases` target against each installed executable. `BIOMCP_TEST_EXECUTABLE` selects the installed executable and `BIOMCP_TEST_FIXTURES` selects the copied tree. Every case directory registers the same Rust trials. Use ordinary feature names to select the cache publication, HTTP and AlphaGenome trials.

Reports include the YAML case count, total trial count and elapsed time. The first complete installed run measures the suite timeout; later installation channels use that measurement plus 15 percent. The platform job bounds the first measurement. Fixture path checks reject direct manifest-directory reads outside the launcher.

The separate `tools` target contains opt-in schema and assertion boundary proofs, parity validation, synthetic adapter projections, generators and live checks. These library or tooling calls do not test the installed executable. Repository ratchets and library contracts remain separate targets. `make library-tests` runs the retained library and private boundary tests. The architecture's test-placement table records their reasons.

## Measure command coverage

`make coverage` runs `cases` against an instrumented product binary. The launcher gives every child a `bin-%m-%p.profraw` profile name. The script merges only those profiles and reports only the actual binary object through LLVM tools. The denominator contains linked product source lines and excludes generated protobuf code. Harness execution and in-process library contracts never count. `make library-tests` validates those libraries separately in the routine gate.

The release maintainer sets `coverage.floor` from the first binary-only measurement and owns subsequent increases. `make unreached` displays line execution counts from the last complete collection. Raw child and harness profiles remain in the coverage target directory for diagnosis until the next collection.

## Inspect a candidate with mutation testing

The release tool manifest pins `cargo-mutants`. The mutation commands run in native jobs. They do not add a runtime dependency. Run `python3 release/mutations.py --help` for a short command list. Each command prints one summary and keeps full native logs beside its plan.

The release owner supplies the previous candidate commit and one shared Unix start time before tool installation. Installation requires a new output directory and keeps its Cargo home and tool prefix there. The output directory must belong to the current job. Add `SETUP/tool/bin` to that job's PATH before planning or running shards. Each separate job needs its own pinned tool environment. A temporary installation ends with its job and cannot supply later shards. Use that same start for `install --started START --output SETUP` and `plan --started START --previous COMMIT --output DIRECTORY`. The plan owns the deadline through `CAP_SECONDS` in the runner. Use a new output directory outside tracked source. Installation, discovery and every shard share the deadline. Queue shards together so queue wait does not consume their budget.

The first plan uses `--measure` to select `CALIBRATION_SIZE` mutants. Candidate plans use `--cost-seconds` from completed viable mutants and `--baseline-seconds` for the measured baseline overhead across their shards on the same machine, toolchain, worker settings and cases suite. The planner reserves elapsed setup time and baseline overhead before choosing the count. The planner considers changed product Rust files before the rule files in `RULE_FILES`. It records the whole inventory and every selected mutant name. Unselected mutants remain in the final unrun list. A cost drawn only from build failures cannot size a candidate run. A surviving mutant may need the whole cases suite even when other mutants fail quickly.

Run `shard --plan DIRECTORY/plan.json --index INDEX` as a separate job for each index declared with the plan's `--shards`. Each shard runs the complete `cargo test -p biomcp --test cases` target with two build and test workers. Each shard also runs its baseline. This command provides no baseline-skip option. It stops the native child process group when the shared deadline expires and keeps partial native outcomes.

Before compiling, each shard discovers its filtered scope using selected filenames and exact mutant names. It compares the complete native identities with the plan's saved inventory. A mismatch stops the shard. The pinned tool can ignore name filters for struct-field deletions, so the filename filter and this comparison both matter. Keep `inventory.json` beside `plan.json` when copying a plan to another job.

The native command selects `biomcp` in its shared Cargo arguments as well as its test-package option. The pinned tool chooses baseline packages from mutated source packages, which may have no `cases` target. The shared Cargo argument includes the owning package in both baseline and mutation phases.

Run `collect --plan DIRECTORY/plan.json` after the shard jobs finish. Attach `mutation-report.json` and the raw shard directories to the candidate. The report separates caught mutants, survivors, build failures, native timeouts and unrun mutants. It preserves native phase durations and reports the number that actually completed. Missing shards, baseline failures and tool errors fail collection. Survivors remain advisory. The release owner chooses at most ten survivors for new outside-in cases, starting with rule modules, and lists the rest.

The release rehearsal exposes each mutation action through `make release-mutations MUTATION_REPORT=REPORT MUTATION_ARGS='ACTION ARGUMENTS'`. Supply a job-owned stage report for each invocation. Submit each planned shard as a separate native job; after all shards finish, invoke `collect` through the same target. This entry point records the action's exit status and leaves its native logs and survivor report beside the plan.

A calibration that hits the cap before all selected mutants finish is incomplete. Do not call it a measured calibration or replace missing results with estimates from booked cores. The full cases duration may prevent the required calibration or assertion-deletion experiment from fitting the cap. The release owner must resolve that limit before running either experiment. A planted proof must use the same exact `status.rs` mutant with one owning assertion deleted and restored. Other assertions may also catch the mutant; check that premise before claiming a survivor.

## Publish an approved release

The tag workflow qualifies and stages the candidate before each channel job requests approval in the `release-publication` environment. The release operator approves those requests after the release owner approves publication. PyPI publication follows the container push; Homebrew and installer publication follow the public release assets; MCP Registry publication follows PyPI. The final install jobs require `biomcp version` from every public channel to match the tag. After all installs pass, a separate `release-publication` approval permits promotion of GitHub and GHCR latest pointers. Promotion copies the attested container digest and compares every published stable release; an older release or a failed release query leaves both pointers unchanged. A failed publication can leave some channels published. Inspect the failed job before retrying; PyPI versions cannot be replaced.

The two latest updates cannot form one transaction. GHCR updates first. If the GitHub update fails, the job fails and GHCR may already point to the candidate. Inspect the public GHCR latest digest, GitHub latest version and the failed run's candidate digest before retrying. Rerun only the failed promotion job from the same run. It rechecks every published stable version before updating either pointer. If a newer stable release now exists, recover through that newer release's checked promotion job. Never roll either pointer back blindly.

Before publication, the repository administrator must configure these external settings after approval:

- Create `release-publication` in `genomoncology/biomcp` with required reviewers, self-review prevention, admin bypass disabled and deployment rules that admit release tags. Every `publish-*` job uses this environment.
- Add a PyPI Trusted Publisher for project `biomcp-cli`, owner `genomoncology`, repository `biomcp`, workflow `release.yml`, environment `release-publication`. The workflow uses GitHub OIDC and uploads PyPI attestations. It needs no PyPI token. See [PyPI's publisher instructions](https://docs.pypi.org/trusted-publishers/adding-a-publisher/).
- Permit the repository's `GITHUB_TOKEN` to publish the public GHCR package `ghcr.io/genomoncology/biomcp` and its attestations. Link an existing package to this repository and grant it Actions access. Versioned images use both staged architectures and the attestation identifies the pushed manifest digest. See [GitHub's container attestation instructions](https://github.com/actions/attest#container-image).
- Store `HOMEBREW_TAP_TOKEN` as an environment secret with Contents read/write access only to `genomoncology/homebrew-biomcp`. Its holder must be allowed to update `main`; the job commits only `Formula/biomcp.rb`.
- Retain the public repository's `gh-pages` branch and configure Pages to serve its root at `biomcp.org`. Permit the workflow token to update that branch and request Pages builds. The installer job commits only `install.sh`; it preserves the existing site. Release assets supply the archives that the installer downloads.
- Permit MCP Registry GitHub OIDC publication for namespace `io.github.genomoncology`. The registry job installs the pinned publisher after checking its checksum and Sigstore signature, then publishes the staged `server.json`. It needs no registry token. See the [registry automation instructions](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/github-actions.mdx).

An approval rehearsal requires a separately approved public fork with the same protected environment. Dispatch `release.yml` from the reviewed candidate with `approval_rehearsal=true`. Qualification still runs. Each channel reaches its environment approval before its job starts. Approve predecessor rehearsal jobs to expose the dependent channel requests, then leave the last requests pending as evidence. After all channel approvals, the dispatch skips public install commands and exposes the final promotion approval. This exercises approval wiring without proving a public installation. Every publication and channel authentication step requires a production tag push, so dispatch never uploads or changes a channel. Record the workflow revision and run URL. Creating the fork, changing its environment and running Actions require separate approval.
