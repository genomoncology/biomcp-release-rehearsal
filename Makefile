.DEFAULT_GOAL := landing-gate

.PHONY: markdown-help-installed-check markdown-help-check markdown-help-gate
markdown-help-installed-check:
	# Historical help is captured separately; strict schema proofs use the matching build.
	python3 scripts/markdown_help_probe.py "$(MARKDOWN_HELP_BASELINE)" target/markdown-help-baseline-evidence
	mkdir -p target/markdown-help
	cargo build --locked -p biomcp --bin biomcp --test tools --message-format=json-render-diagnostics > target/markdown-help/build.jsonl
	python3 -c 'import hashlib,json,subprocess,sys; from pathlib import Path; rows=[json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]; artifacts={row["target"]["name"]:row for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable")}; executables={name:artifacts[name]["executable"] for name in ("biomcp","tools")}; binding={"revision":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"artifacts":{name:{"sha256":hashlib.sha256(Path(path).read_bytes()).hexdigest(),"profile":artifacts[name]["profile"]} for name,path in executables.items()}}; Path("target/markdown-help/source-bindings.json").write_text(json.dumps(binding,indent=2)+"\n"); print(json.dumps(binding),flush=True); subprocess.run([sys.executable,"scripts/trials.py","--target","tools","--proofs","--runner",executables["tools"],"--executable",executables["biomcp"],"--installed","schema-discovery/rejects-bad-cases"],check=True)' target/markdown-help/build.jsonl

markdown-help-check: markdown-help-installed-check
	cargo test --locked -p biomcp --test cases help $(TEST_THREADS)
	+$(MAKE) check-cheap

markdown-help-gate: markdown-help-installed-check
	+$(MAKE) landing-gate

.PHONY: article-default-prepare article-default-focused article-default-lint
article-default-prepare:
	cargo fmt --all
	+$(MAKE) schema-generate docs-generate
	mkdir -p target/article-default-generated
	tar -czf target/article-default-generated/source.tar.gz crates/biomcp-types/schemas docs/reference/commands.md

article-default-focused:
	rm -rf target/article-default-proof
	mkdir -p target/article-default-proof
	BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/article-default-proof" cargo test --locked -p biomcp --test tools -- --cases "$(CURDIR)/tests/tool-cases" article/
	cargo test --locked -p biomcp --test cases grammar/argument-description-guidance
	cargo test --locked -p biomcp --test cases grammar/numeric-description-guidance
	cargo test --locked -p biomcp --test cases trial/input-boundaries
	cargo test --locked -p biomcp --test cases variant/identity
	cargo test --locked -p biomcp --test cases article/fulltext/
	cargo test --locked -p biomcp --test cases article/preprint/

article-default-lint: fmt
	cargo clippy --keep-going --locked -p biomcp-types -p biomcp-query -p biomcp --all-targets --features biomcp-types/schema -- -D warnings
	+$(MAKE) current-schema-check
	cargo test --locked -p biomcp --test cases article/help
	cargo test --locked -p biomcp --test cases article/get/

.PHONY: article-default-corrected-check
article-default-corrected-check: article-default-lint
	cargo test --locked -p biomcp --test ratchets repository_obeys_structural_contract -- --exact
	cargo test --locked -p biomcp --test cases query-validation/argument-recovery -- --exact
	+$(MAKE) article-default-focused

.PHONY: drug-alias-review-regressions drug-alias-review-tools
drug-alias-review-regressions:
	cargo test --locked -p biomcp --test cases drug/source-aliases-review
	$(MAKE) drug-alias-review-tools

drug-alias-review-tools:
	cargo test --locked -p biomcp --test tools -- --cases "$(CURDIR)/tests/tool-cases" drug-native-sources/drug/source-aliases-review

.PHONY: drug-alias-baseline
drug-alias-baseline: plain-cli-build
	python3 scripts/drug_alias_probe.py

.PHONY: drug-alias-prepare drug-alias-check drug-alias-review-check drug-alias-scans
DRUG_ALIAS_BASELINE ?= d590e1c7bf92fbcf8b0dc299814d9fe2983244b5
drug-alias-prepare:
	cargo fmt --all
	$(MAKE) schema-generate docs-generate
	mkdir -p dist
	COPYFILE_DISABLE=1 tar -czf dist/drug-alias-generated.tar.gz crates/biomcp-query/src/drug.rs crates/biomcp-query/src/drug/aliases.rs crates/biomcp-query/src/drug/resolve.rs crates/biomcp-types/src/drug/query.rs crates/biomcp-types/src/drug/search.rs crates/biomcp-types/schemas docs/reference/commands.md
	$(MAKE) drug-alias-check

drug-alias-check:
	cargo clippy --keep-going --locked -p biomcp-types -p biomcp-query -p biomcp --all-targets --features biomcp-types/schema -- -D warnings
	cargo test --locked -p biomcp --test cases drug/
	cargo test --locked -p biomcp --test tools -- --cases "$(CURDIR)/tests/tool-cases" drug-native-sources/drug/source-aliases.yaml --exact

drug-alias-review-check: drug-alias-check drug-alias-review-tools current-schema-check schema-release-check
	$(MAKE) drug-alias-scans

drug-alias-scans:
	sh scripts/check_trial_results.sh scans $(DRUG_ALIAS_BASELINE)

.PHONY: drug-alias-main-integration
drug-alias-main-integration:
	cargo clippy --keep-going --locked -p biomcp-types -p biomcp-query -p biomcp --all-targets --features biomcp-types/schema -- -D warnings
	cargo test --locked -p biomcp --test cases drug/source-aliases
	cargo test --locked -p biomcp --test cases drug/indication.yaml
	$(MAKE) drug-alias-review-tools current-schema-check

.PHONY: drug-alias-cache-regressions
drug-alias-cache-regressions:
	cargo test --locked -p biomcp --test cases drug-native-sources/chembl-search-cache-admission -- --exact
	cargo test --locked -p biomcp --test cases drug-native-sources/mychem-search-cache-admission -- --exact

.PHONY: drug-label-alias-integration
drug-label-alias-integration: fmt
	cargo clippy --keep-going --locked -p biomcp-adapters -p biomcp-types -p biomcp-query -p biomcp --all-targets --features biomcp-types/schema -- -D warnings
	cargo test --locked -p biomcp --test cases drug/source-aliases $(TEST_THREADS)
	cargo test --locked -p biomcp --test tools -- --cases "$(CURDIR)/tests/tool-cases" drug-native-sources/drug/source-aliases
	cargo test --locked -p biomcp --test cases drug/indication.yaml $(TEST_THREADS)
	$(MAKE) label-priority-check label-display-check

.PHONY: drug-source-guidance-check
drug-source-guidance-check:
	+$(MAKE) schema-generate docs-generate
	+$(MAKE) --keep-going typed-schema-check typed-hosted-check

.PHONY: faers-counts-integration-check
faers-counts-integration-check:
	sh scripts/faers_counts_job.sh mechanical
	sh scripts/faers_counts_job.sh focused
	+$(MAKE) current-schema-check

# Cargo and the test harnesses do not interpret pm's fractional core booking.
ifneq ($(strip $(PM_JOB_CORES)),)
PM_WORKERS := $(shell python3 -c 'import math,os; print(max(1, math.floor(float(os.environ["PM_JOB_CORES"]))))')
ifeq ($(PM_WORKERS),)
$(error PM_JOB_CORES must be a finite number)
endif
export CARGO_BUILD_JOBS ?= $(PM_WORKERS)
export RUST_TEST_THREADS ?= $(PM_WORKERS)
endif
TEST_THREADS = $(if $(strip $(RUST_TEST_THREADS)),-- --test-threads $(RUST_TEST_THREADS))

.PHONY: landing-gate full-suite plain-cli-build fmt lint ratchets test audit coverage unreached

landing-gate:
	+python3 scripts/landing_gate.py

full-suite:
	+python3 scripts/landing_gate.py --full

# Keep these sequential even when the caller uses make -j.
.PHONY: pre-job check-cheap proof-check proof-list proof proof-unpack gate-stage-check
pre-job: check-cheap

LINT_GOALS = unused-dependencies check-no-default-features lint-workspace lint-tools lint-rustdoc

check-cheap:
	python3 scripts/check_docs.py
	+$(MAKE) --no-print-directory --keep-going -j1 shell-check fmt $(LINT_GOALS) ratchets

.PHONY: shell-check
shell-check:
	python3 scripts/check_shell.py

# Retained proof archives use the platform tar, including Python 3.9 machines.
proof-unpack:
	@test -n "$(ARCHIVE)" -a -n "$(OUT)" || (echo 'Set ARCHIVE and OUT'; exit 1)
	mkdir -p "$(OUT)"
	tar -xf "$(ARCHIVE)" -C "$(OUT)"

# Select literal names printed by proof-list, for example make proof PROOFS='name'.
proof-list: pre-job
	python3 scripts/trials.py --target tools --proofs --list

proof: pre-job
	python3 scripts/trials.py --target tools --proofs $(PROOFS)

# Reuse the committed proof discovery and comparison checks.
proof-check:
	python3 scripts/test_trials.py
	python3 scripts/test_reply_diff.py
	python3 scripts/test_schema_release.py
	+$(MAKE) --no-print-directory gate-stage-check

gate-stage-check:
	python3 -m unittest scripts.test_gate_stages

plain-cli-build:
	cargo build --locked -p biomcp --bin biomcp --timings

fmt:
	cargo fmt --all -- --check

.PHONY: $(LINT_GOALS)
lint:
	+$(MAKE) --no-print-directory --keep-going -j1 $(LINT_GOALS)

check-no-default-features:
	cargo check --keep-going --locked -p biodata -p biomcp-types --no-default-features

lint-workspace:
	cargo clippy --keep-going --workspace --all-targets --locked -- -D warnings

lint-tools:
	cargo clippy --keep-going --locked -p biomcp --test tools -- -D warnings

lint-rustdoc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --keep-going

ratchets: plain-cli-build
	cargo test --locked -p biomcp --test ratchets $(TEST_THREADS)

test:
	cargo test --workspace --locked $(TEST_THREADS)

.PHONY: public-dump-check
public-dump-check:
	cargo build --locked -p biomcp-types --example public_dump_values
	PUBLIC_DUMP_VALUES="$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/examples/public_dump_values" python3 scripts/test_public_dump_sweep.py

audit:
	cargo deny --locked check licenses advisories bans sources

.PHONY: cases
cases:
	cargo test --locked -p biomcp --test cases $(TEST_THREADS)

.PHONY: rights
rights:
	python3 scripts/article_licenses.py
	python3 scripts/test_article_licenses.py

.PHONY: library-tests
library-tests:
	cargo test --workspace --exclude biomcp --features biomcp-types/schema --locked $(TEST_THREADS)
	cargo test --locked -p biomcp --bin biomcp $(TEST_THREADS)

coverage: schema-shape-check
	sh scripts/coverage.sh run

.PHONY: schema-shape-check
schema-shape-check: docs-check library-tests
	cargo test --locked -p biomcp --test cases mcp-research/research-registry-projection-paths $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-research/research-search-fetch-parity $(TEST_THREADS)

unreached:
	sh scripts/coverage.sh unreached

.PHONY: docs-check
docs-check:
	cargo test --locked -p biomcp --test cases readme/ $(TEST_THREADS)
	cargo test --locked -p biomcp --bin biomcp grammar::reference::reference_matches_grammar $(TEST_THREADS)
	python3 release/toolchain.py check
	python3 -m unittest release.test_toolchain
	python3 release/tools.py
	python3 -m unittest release.test_tools release.test_linux release.test_mutations
	python3 scripts/check_docs.py
	python3 scripts/check_front_map.py "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp"
	+$(MAKE) --no-print-directory rights
	python3 scripts/test_docs.py
	python3 scripts/test_source_capture.py
	python3 scripts/test_source_capture_plan.py
	python3 scripts/test_coverage.py
	python3 scripts/test_reply_diff.py
	python3 scripts/test_schema_release.py
	+$(MAKE) --no-print-directory gate-stage-check
	python3 scripts/test_build_workers.py
	python3 scripts/test_trials.py
	python3 scripts/test_cache_process_load.py
	python3 scripts/test_nightly_live_check.py
	python3 -m unittest release.test_installer release.test_checks release.test_gate release.test_windows release.test_artifacts.ReleaseCommands
	python3 -m unittest release.test_workflow release.test_security release.test_licenses release.test_docs
	python3 -m unittest release.test_source_revision release.test_workflow_contracts release.test_publication release.test_promotion
.PHONY: release-linux
release-linux:
	sh release/linux.sh $(CONTAINER_ARGS)

.PHONY: release-artifacts
release-artifacts:
	sh release/matrix.sh $(CONTAINER_ARGS)

.PHONY: nightly-live-check
nightly-live-check:
	python3 scripts/nightly_live_check.py

.PHONY: source-capture
source-capture:
	python3 scripts/source_capture.py --plan "$(PLAN)" --out "$(OUT)"

.PHONY: docs-generate
docs-generate:
	cargo test --locked -p biomcp --bin biomcp grammar::reference::write_reference $(if $(strip $(RUST_TEST_THREADS)),-- --test-threads $(RUST_TEST_THREADS) --ignored,-- --ignored)
.PHONY: schema-generate schema-release-check
schema-generate:
	python3 scripts/schema_release.py --generate

schema-release-check:
	python3 scripts/schema_release.py

.PHONY: current-schema-check
current-schema-check:
	cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types -- --exact

.PHONY: tensor-proto-check
tensor-proto-check:
	python3 scripts/regenerate_tensor_proto.py --check

.PHONY: secrets
secrets:
	python3 release/secrets.py history . --since "$(SECRET_SCAN_BASE)"

# This ancestor was covered by the retained j24209 full-history scan.
# Advance only after the replacement ancestor has passed secret screening.
SECRET_SCAN_BASE ?= $(shell cat release/secrets-main)
.PHONY: secrets-full
secrets-full:
	python3 release/secrets.py history .

COLD_REPORT ?= $(CURDIR)/dist/cold-report.json

.PHONY: release-cold
release-cold:
	python3 release/rehearse.py cold --report "$(COLD_REPORT)" -- $(CONTAINER_ARGS)

.PHONY: release-mutations
release-mutations:
	@test -n "$(MUTATION_REPORT)" || (echo 'Set MUTATION_REPORT to a job-owned stage report'; exit 1)
	python3 release/rehearse.py mutation --report "$(MUTATION_REPORT)" -- $(MUTATION_ARGS)

.PHONY: workflow-lint
workflow-lint:
	cargo check --locked -p biodata -p biomcp-types --no-default-features
	actionlint

.PHONY: publication-check
publication-check:
	actionlint
	python3 -m unittest release.test_publication release.test_workflow release.test_workflow_contracts release.test_toolchain release.test_promotion
	python3 release/toolchain.py check
	python3 scripts/check_docs.py
.PHONY: unused-dependencies dependency-graph dependency-proof
unused-dependencies:
	python3 release/toolchain.py install-machete
	cargo machete

dependency-graph:
	cargo tree --locked -p biomcp --edges normal,build
	cargo tree --locked --target all --edges normal,build --prefix none --format "{p}"

# Deliberately add an unused dependency and require the real lint entry point to fail.
dependency-proof:
	python3 -m unittest release.test_toolchain
	python3 scripts/dependency_proof.py
.PHONY: bundle-tests
bundle-tests:
	cargo test --locked -p biomcp-fetch --test bundle $(TEST_FILTER) $(TEST_THREADS)

.PHONY: bundle-checks
bundle-checks: fmt
	cargo clippy --locked -p biomcp-fetch --all-targets -- -D warnings
	$(MAKE) ratchets
.PHONY: cli-fixes-check
cli-fixes-check: fmt lint ratchets
	$(MAKE) schema-generate docs-generate
	mkdir -p dist
	COPYFILE_DISABLE=1 tar -czf dist/cli-generated.tar.gz crates/biomcp-types/schemas docs/reference/commands.md
	cargo test --locked -p biomcp --bin biomcp grammar::reference::reference_matches_grammar $(TEST_THREADS)
	python3 scripts/check_docs.py
	$(MAKE) cli-fixes-cases

CLI_FIX_CASES ?= local/ key-registration/ article-saves/ trial/name-fallback-continuation trial/input-boundaries author/help author/papers-help author/source-alias cell-line/aliases cell-line/examples mcp-parity/discovers-artifact-command-tree

.PHONY: cli-fixes-cases
cli-fixes-cases:
	@set -e; for selection in $(CLI_FIX_CASES); do \
		cargo test --locked -p biomcp --test cases "$$selection" $(TEST_THREADS); \
	done
.PHONY: http-edge-check
http-edge-check: fmt
	cargo clippy --locked -p biomcp --all-targets -- -D warnings
	cargo test --locked -p biomcp --test ratchets $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-http/ $(TEST_THREADS)

# Long suite: real-time and multi-process checks repeated under load, run at final
# release and for touched checks: make stress STRESS_ARGS="--only fetch/retries".
.PHONY: stress
stress:
	mkdir -p target/stress
	cargo build --locked -p biomcp -p biomcp-fetch --bin biomcp --test cases --test tools --test cache --test retries --test transport --test quotas --test grpc_worker --test grpc_persistence --test bundle --message-format=json > target/stress/build.jsonl
	python3 scripts/cache_process_load.py target/stress/build.jsonl $(STRESS_ARGS)

.PHONY: hosted-http-load
hosted-http-load:
	cargo test --locked -p biomcp --bin biomcp hosted_http_sync_load -- --ignored --nocapture

.PHONY: cli-candidate-check
cli-candidate-check: fmt
	cargo clippy --locked -p biomcp --test cases -- -D warnings
	$(MAKE) ratchets
	python3 -c 'import json,subprocess,sys; out=subprocess.run([sys.argv[1], "mcp-config", "--client", "mcp-servers", "--absolute-path", "--offline"],capture_output=True,check=False); print("REMOVED_CONFIG_OPTION bytes="+str(len(out.stdout))+" exit="+str(out.returncode)); print(out.stdout[:1024].decode()); assert out.returncode == 2; assert json.loads(out.stdout)["error"]["code"] == "usage.arguments"' "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp"
	$(MAKE) cli-fixes-cases CLI_FIX_CASES='trial/ housekeeping/ local/mcp-config local/removed-options local/key-list-help'

.PHONY: test-support-checks
test-support-checks: pre-job
	cargo test --locked -p biomcp-fetch --lib stream_worker_shutdown_cancels_and_joins_active_network_work $(TEST_THREADS)
	cargo test --locked -p biomcp-types --test response $(TEST_THREADS)
	cargo test --locked -p biomcp-fetch --test alphagenome --test grpc_worker --test grpc_persistence --test bundle $(TEST_THREADS)
	cargo test --locked -p biomcp-adapters --test mapped_refusals --test custom_refusals --test ctgov_refusals $(TEST_THREADS)
	cargo test --locked -p biomcp-query --test article_search --test gene_resolution --test query_refusals --test collection_refusals $(TEST_THREADS)
	python3 scripts/trials.py --target tools --proofs schema-discovery/terms-subtraction-keeps-payload-ceilings schema-discovery/markdown-terms-subtraction-keeps-payload-ceilings schema-discovery/capture-retains-replay-owner schema-discovery/persistent-fixture-evidence
	python3 scripts/trials.py --target cases variant-alphagenome/alphagenome-source-identity-refusals

.PHONY: public-docs-check
public-docs-check:
	+$(MAKE) --no-print-directory pre-job
	+$(MAKE) --no-print-directory docs-check

.PHONY: http-busy-diagnosis
http-busy-diagnosis: fmt
	cargo test --locked -p biomcp --test cases mcp-http/framing-resource-deadlines-and-shutdown $(TEST_THREADS)
	cargo llvm-cov test --no-report --locked -p biomcp --test cases mcp-http/framing-resource-deadlines-and-shutdown $(TEST_THREADS)

.PHONY: http-edge-coverage
http-edge-coverage:
	cargo llvm-cov test --no-report --locked -p biomcp --test cases mcp-http/ $(TEST_THREADS)
	cargo llvm-cov test --no-report --locked -p biomcp --test cases mcp-hosted/ $(TEST_THREADS)
	cargo llvm-cov test --no-report --locked -p biomcp --test cases key-registration/ $(TEST_THREADS)
	cargo llvm-cov test --no-report --locked -p biomcp --test tools -- --proofs schema-discovery/isolates-child-environments $(if $(strip $(RUST_TEST_THREADS)),--test-threads $(RUST_TEST_THREADS))

.PHONY: http-capacity-check
http-capacity-check: pre-job docs-check
	cargo test --locked -p biomcp --test cases mcp-http/ $(TEST_THREADS)

.PHONY: mcp-reply-focused mcp-reply-gate
mcp-reply-focused mcp-reply-gate: export PATH := /Users/ian/.local/share/uv/python/cpython-3.13.17-macos-aarch64-none/bin:$(HOME)/.cargo/bin:$(PATH)
mcp-reply-focused mcp-reply-gate: export RUSTUP_TOOLCHAIN := 1.95.0
mcp-reply-focused: export BIOMCP_TRANSCRIPT_DIR := $(CURDIR)/target/0587-proofs/replies
mcp-reply-focused:
	cargo fmt --all -- --check
	mkdir -p target/mcp-reply/installed
	cargo install --locked --debug --path crates/biomcp --root target/mcp-reply/installed --force
	BIOMCP_TEST_EXECUTABLE="$(CURDIR)/target/mcp-reply/installed/bin/biomcp" cargo test --locked -p biomcp --test cases mcp-parity/ $(TEST_THREADS)
	BIOMCP_TEST_EXECUTABLE="$(CURDIR)/target/mcp-reply/installed/bin/biomcp" cargo test --locked -p biomcp --test cases mcp-http/ $(TEST_THREADS)
	BIOMCP_TEST_EXECUTABLE="$(CURDIR)/target/mcp-reply/installed/bin/biomcp" cargo test --locked -p biomcp --test cases mcp-hosted/ $(TEST_THREADS)
	python3 scripts/check_front_map.py target/mcp-reply/installed/bin/biomcp --mcp-baseline target/0587-proofs/baseline-installed/biomcp --output target/0587-proofs/candidate
mcp-reply-gate: landing-gate

JSCPD ?= jscpd
.PHONY: mcp-reply-scan
mcp-reply-scan:
	$(JSCPD) --version
	$(JSCPD) crates/biomcp/src/config.rs crates/biomcp/src/config crates/biomcp/src/help.rs crates/biomcp/src/mcp.rs crates/biomcp/src/mcp --format rust --max-size 10mb --reporters json --output target/0587-proofs/jscpd-product
	$(JSCPD) crates/biomcp/tests/runner/protocol.rs crates/biomcp/tests/runner/protocol_schemas.rs crates/biomcp/tests/runner/mcp_reply_form.rs --format rust --max-size 10mb --reporters json --output target/0587-proofs/jscpd-tests

.PHONY: civic-evidence-build
civic-evidence-build:
	cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json

.PHONY: civic-evidence-focused
civic-evidence-focused:
	+sh scripts/check_civic_evidence.sh focused

.PHONY: civic-evidence-structure
civic-evidence-structure:
	cargo test --locked -p biomcp --test ratchets repository_obeys_structural_contract -- --exact $(if $(strip $(RUST_TEST_THREADS)),--test-threads $(RUST_TEST_THREADS))

.PHONY: civic-evidence-combined
civic-evidence-combined:
	+sh scripts/check_civic_evidence.sh combined

.PHONY: typed-reply-check
typed-reply-check:
	mkdir -p "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-replies"
	cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json-render-diagnostics > "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-replies/build.jsonl"
	python3 -c 'import json,subprocess,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; executables={row["target"]["name"]:row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable")}; subprocess.run([sys.executable,"scripts/trials.py","--runner",executables["cases"],"--executable",executables["biomcp"],"--installed","mcp-parity/typed-reply-forms-and-cap","mcp-parity/complete-tool-result-character-cap"],check=True)' "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-replies/build.jsonl"

.PHONY: typed-schema-check
typed-schema-check:
	mkdir -p "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-schemas"
	cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json-render-diagnostics > "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-schemas/build.jsonl"
	python3 -c 'import json,subprocess,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; executables={row["target"]["name"]:row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable")}; subprocess.run([sys.executable,"scripts/trials.py","--runner",executables["cases"],"--executable",executables["biomcp"],"--installed","mcp-parity/compact-typed-schemas"],check=True)' "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-schemas/build.jsonl"

.PHONY: typed-schema-source-check
typed-schema-source-check:
	+$(MAKE) schema-generate docs-generate
	+$(MAKE) --keep-going fmt typed-schema-lint typed-schema-metadata-lint typed-schema-check typed-schema-structure

.PHONY: typed-schema-lint typed-schema-metadata-lint typed-schema-structure
typed-schema-lint:
	cargo clippy --locked -p biomcp --bin biomcp --test cases -- -D warnings
typed-schema-metadata-lint:
	cargo clippy --locked -p biomcp-types --features schema --test schemas -- -D warnings
typed-schema-structure:
	cargo test --locked -p biomcp --test ratchets $(TEST_THREADS)

.PHONY: typed-help-check
typed-help-check:
	mkdir -p "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-help"
	cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json-render-diagnostics > "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-help/build.jsonl"
	python3 -c 'import json,subprocess,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; executables={row["target"]["name"]:row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable")}; subprocess.run([sys.executable,"scripts/trials.py","--runner",executables["cases"],"--executable",executables["biomcp"],"--installed","mcp-parity/typed-help-research-discovery","mcp-parity/typed-help-next-recovery"],check=True)' "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-help/build.jsonl"


# Local bootstrap proof only; no Windows VM or candidate qualification.
.PHONY: windows-bootstrap-check windows-bootstrap-walk
windows-bootstrap-check: shell-check
	uvx --offline --from ruff==0.17.2 ruff check --select F401,F841,ERA,BLE,E722,S110 release/gate.py release/test_gate.py
	uvx --offline --from vulture==2.14 vulture release/gate.py --min-confidence 60
	python3 -m unittest release.test_gate release.test_source_revision release.test_windows

windows-bootstrap-walk: windows-bootstrap-check
	BIOMCP_TEST_ARCHIVE_BOOTSTRAP=1 python3 -m unittest release.test_gate.InstalledCli.test_archive_bootstrap_preserves_tooling_and_product_identity


.PHONY: typed-hosted-check
typed-hosted-check:
	mkdir -p "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-hosted"
	cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json-render-diagnostics > "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-hosted/build.jsonl"
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/typed-hosted/catalogs" python3 -c 'import json,subprocess,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; executables={row["target"]["name"]:row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable")}; subprocess.run([sys.executable,"scripts/trials.py","--runner",executables["cases"],"--executable",executables["biomcp"],"--installed","mcp-parity/typed-hosted-catalog"],check=True)' "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/typed-hosted/build.jsonl"

.PHONY: hosted-surface-check
hosted-surface-check:
	python3 scripts/trials.py --installed mcp-hosted/served-surface-and-source-policy mcp-parity/typed-hosted-catalog

.PHONY: hosted-surface-mechanical
hosted-surface-mechanical:
	cargo fmt --all -- --check
	+$(MAKE) hosted-surface-lint
	cargo test --locked -p biomcp --test ratchets -- repository_obeys_structural_contract --exact

.PHONY: hosted-surface-development
hosted-surface-development:
	+$(MAKE) --keep-going hosted-surface-check hosted-surface-mechanical

.PHONY: hosted-surface-generate
hosted-surface-generate:
	+$(MAKE) schema-generate docs-generate
	tar -czf target/hosted-surface-generated.tar.gz crates/biomcp-types/schemas docs/reference/commands.md
.PHONY: drug-indication-focused
drug-indication-focused:
	+$(MAKE) schema-generate docs-generate
	mkdir -p target/drug-indication-generated
	tar -czf target/drug-indication-generated/source.tar.gz crates/biomcp-types/schemas docs/reference/commands.md
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/drug-indication-transcripts" python3 scripts/trials.py --installed mcp-parity/typed-cli-bridge mcp-parity/compact-typed-schemas mcp-parity/typed-hosted-catalog --case-file drug/indication.yaml --case-file drug/openfda-label-page.yaml --case-file drug/label-free-text.yaml --case-file drug/retired.yaml

.PHONY: source-status-check
source-status-check:
	cargo test --locked -p biomcp-types --test rust_api $(TEST_THREADS)
	cargo test --locked -p biomcp-types --doc $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases source-status $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases variant-hotspots/ $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases diagnostic/fda-cdx $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases local/commands $(TEST_THREADS)

.PHONY: source-status-pre-review
source-status-pre-review:
	+$(MAKE) source-status-check check-cheap current-schema-check schema-release-check

.PHONY: deadline-setup-focused deadline-setup-load
deadline-setup-focused: fmt lint-tools
	python3 scripts/deadline_setup_load.py

deadline-setup-load:
	python3 scripts/deadline_setup_load.py load

.PHONY: bench-honesty-repro
bench-honesty-repro:
	cargo build --locked -p biomcp --bin biomcp
	python3 scripts/bench_honesty.py "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp"

.PHONY: bench-honesty-focused bench-honesty-cases bench-honesty-tools bench-honesty-generate bench-honesty-mechanical bench-honesty-lint
bench-honesty-focused:
	+$(MAKE) --keep-going bench-honesty-cases bench-honesty-tools bench-honesty-mechanical
bench-honesty-cases:
	python3 scripts/trials.py --installed query-validation/argument-recovery --case-file adverse-event/drug-boolean-refusal.yaml --case-file adverse-event/faers-literals-and-cap.yaml --case-file adverse-event/faers-several-names.yaml --case-file variant/resolver-provenance.yaml --case-file variant/resolver-variantvalidator.yaml --case-file diagnostic/device-approvals.yaml --case-file gene/sections/clingen-validity.yaml
bench-honesty-tools:
	cargo test --locked -p biomcp --test tools -- --cases "$(CURDIR)/tests/tool-cases" adverse-event-search/adverse-event/drug-boolean-refusal.yaml --exact
bench-honesty-generate:
	cargo fmt --all
	$(MAKE) schema-generate docs-generate
bench-honesty-mechanical:
	+$(MAKE) --keep-going fmt bench-honesty-lint ratchets docs-check current-schema-check
bench-honesty-lint:
	cargo clippy --keep-going --locked -p biomcp-types -p biomcp-query -p biomcp --all-targets --features biomcp-types/schema -- -D warnings

.PHONY: bench-honesty-recheck bench-honesty-case-recheck
bench-honesty-recheck:
	+$(MAKE) --keep-going bench-honesty-case-recheck bench-honesty-tools fmt bench-honesty-lint ratchets
bench-honesty-case-recheck:
	cargo test --locked -p biomcp --test cases drug-boolean-refusal $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases resolver-provenance $(TEST_THREADS)

.PHONY: bench-honesty-selector-focus bench-honesty-population-cases bench-honesty-provenance-cases
bench-honesty-selector-focus:
	+$(MAKE) --keep-going bench-honesty-population-cases bench-honesty-provenance-cases fmt bench-honesty-lint
bench-honesty-population-cases:
	cargo test --locked -p biomcp --test cases variant-population/ $(TEST_THREADS)
bench-honesty-provenance-cases:
	cargo test --locked -p biomcp --test cases resolver-provenance $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases resolver-variantvalidator $(TEST_THREADS)

.PHONY: indel-population-check
indel-population-check:
	+$(MAKE) check-cheap
	python3 scripts/trials.py --case-file variant/id-forms/deletion.yaml --case-file variant/id-forms/insertion.yaml --case-file variant/id-forms/delins.yaml --case-file variant/population/indel-native.yaml --case-file variant/population/indel-typed.yaml --case-file variant/population/indel-source-error.yaml --case-file variant/population/indel-invalid-crossref.yaml --case-file variant/population/homozygotes-joint.yaml --case-file variant/population/source-counts.yaml

.PHONY: hosted-surface-annotations
hosted-surface-annotations:
	cargo fmt --all -- --check
	cargo test --locked -p biomcp-types --features schema --test schemas required_component_annotations_are_metadata_with_checked_sources -- --exact
	+$(MAKE) hosted-surface-lint

.PHONY: hosted-surface-lint
hosted-surface-lint:
	cargo clippy --keep-going --locked -p biomcp -p biomcp-types --all-targets --all-features -- -D warnings

.PHONY: hosted-reply-parity hosted-reply-parity-check
hosted-reply-parity:
	+$(MAKE) --keep-going hosted-reply-parity-check hosted-surface-lint fmt

hosted-reply-parity-check:
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/hosted-reply-parity/$(shell git rev-parse HEAD)" python3 scripts/trials.py --installed mcp-parity/reply-forms mcp-parity/typed-reply-forms-and-cap mcp-hosted/use/source-admission mcp-hosted/served-surface-and-source-policy

.PHONY: supplement-indications-reproduce
supplement-indications-reproduce:
	python3.13 scripts/check_supplement_indications.py

.PHONY: supplement-indications-focused
supplement-indications-focused:
	+$(MAKE) schema-generate
	mkdir -p target/supplement-indications
	tar -czf target/supplement-indications/schema.tar.gz crates/biomcp-types/schemas/drug-approvals.json
	python3.13 scripts/trials.py --installed --case-file drug/supplement-purpose.yaml --case-file drug/supplement-purpose-missing.yaml --case-file drug/supplement-purpose-mismatch.yaml --case-file drug/supplement-purpose-rejected.yaml drug-approvals/newest-submission-pages drug-approvals/undated-and-numeric-ties

.PHONY: supplement-letter-reproduce
supplement-letter-reproduce:
	python3.13 scripts/check_supplement_indications.py letter

.PHONY: supplement-indications-check
supplement-indications-check:
	+$(MAKE) supplement-indications-focused check-cheap current-schema-check schema-release-check

.PHONY: label-priority-check
label-priority-check:
	BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/label-priority-evidence" cargo test --locked -p biomcp --test cases drug-label/indication-priority-and-native-pages $(TEST_THREADS)
	@for selection in drug/indication.yaml drug/openfda-label-page.yaml drug/label-discovery.yaml drug/label-free-text.yaml drug/label-controls.yaml; do \
		cargo test --locked -p biomcp --test cases "$$selection" $(TEST_THREADS) || exit $$?; \
	done
	cargo test --locked -p biomcp --test cases member-values/openfda-label $(TEST_THREADS)

.PHONY: label-priority-integration
label-priority-integration: fmt label-priority-check label-display-check
	cargo clippy --locked -p biomcp-adapters -p biomcp-query -p biomcp --all-targets -- -D warnings
	cargo test --locked -p biomcp --test ratchets $(TEST_THREADS)
	python3 scripts/check_docs.py

.PHONY: label-display-check
label-display-check:
	BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/label-display-evidence" cargo test --locked -p biomcp --test cases drug-label/native-name-fallback $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases drug/label-display-body.yaml $(TEST_THREADS)

.PHONY: label-display-baseline-check
label-display-baseline-check:
	cd target/label-display-baseline && shasum -a 256 -c binary.sha256
	BIOMCP_TEST_EXECUTABLE="$(CURDIR)/target/label-display-baseline/biomcp" BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/label-display-baseline-evidence" cargo test --locked -p biomcp --test cases drug-label/native-name-fallback $(TEST_THREADS)

.PHONY: help-contract-check
help-contract-check:
	cargo test --locked -p biomcp --bin biomcp every_rendered_flag_retains_its_schema_or_parser_description $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases schema-discovery/help-contract $(TEST_THREADS)

.PHONY: help-metadata-prepare
help-metadata-prepare:
	+$(MAKE) schema-generate docs-generate help-contract-check

.PHONY: help-metadata-final
help-metadata-final:
	+$(MAKE) schema-generate docs-generate help-contract-check
	python3 scripts/refresh_help_cases.py "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp"
	cargo test --locked -p biomcp --test cases help $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-parity/typed-description-contract $(TEST_THREADS)

.PHONY: help-pre-review
help-pre-review:
	+$(MAKE) help-contract-check check-cheap current-schema-check schema-release-check
	cargo test --locked -p biomcp --test cases mcp-parity/typed-description-contract $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-parity/mcp-typed-tools $(TEST_THREADS)

.PHONY: help-alias-check
help-alias-check:
	cargo test --locked -p biomcp --test cases help-entity-aliases $(TEST_THREADS)
	cargo test --locked -p biomcp --test ratchets required_surface_has_asserted_calls $(TEST_THREADS)

.PHONY: help-recovery-check
help-recovery-check:
	+$(MAKE) help-metadata-final check-cheap
	cargo test --locked -p biomcp --test cases description-guidance $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases trial/input-boundaries $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-parity/typed-hosted-catalog $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-parity/mcp-typed-tools $(TEST_THREADS)

.PHONY: caid-population-check
caid-population-check:
	mkdir -p "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/caid-population"
	cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json-render-diagnostics > "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/caid-population/build.jsonl"
	BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/caid-population-evidence" python3 -c 'import json,subprocess,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; executables={row["target"]["name"]:row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable")}; subprocess.run([sys.executable,"scripts/trials.py","--runner",executables["cases"],"--executable",executables["biomcp"],"--installed","--case-file","variant/population/caid-typed.yaml"],check=True)' "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/caid-population/build.jsonl"


.PHONY: typed-description-check typed-description-contract-check
typed-description-check:
	+$(MAKE) --keep-going typed-description-contract-check typed-schema-check typed-hosted-check help-contract-check

typed-description-contract-check:
	cargo test --locked -p biomcp --test cases mcp-parity/typed-description-contract $(TEST_THREADS)

.PHONY: typed-description-lint-check
typed-description-lint-check:
	+$(MAKE) --keep-going fmt lint-workspace lint-tools typed-description-contract-check


.PHONY: typed-followups-check typed-followups-affected
typed-followups-check:
	BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/typed-followups" cargo test --locked -p biomcp --test cases mcp-parity/typed-followups $(TEST_THREADS)
typed-followups-affected:
	+$(MAKE) --keep-going check-cheap typed-followups-parity typed-followups-toolcases
.PHONY: typed-followups-parity typed-followups-toolcases typed-followups-aliases typed-followups-faers
typed-followups-parity typed-followups-toolcases: export BIOMCP_TRANSCRIPT_DIR := $(CURDIR)/target/typed-followups-affected
typed-followups-parity:
	cargo test --locked -p biomcp --test cases mcp-parity/ $(TEST_THREADS)
typed-followups-toolcases:
	+$(MAKE) --keep-going typed-followups-aliases typed-followups-faers
typed-followups-aliases:
	cargo test --locked -p biomcp --test tools -- --cases ../../tests/tool-cases source-aliases $(filter-out --,$(TEST_THREADS))
typed-followups-faers:
	cargo test --locked -p biomcp --test tools -- --cases ../../tests/tool-cases faers-reaction-counts.yaml $(filter-out --,$(TEST_THREADS))


.PHONY: typed-followups-recovery typed-followups-recovery-check
typed-followups-recovery: export BIOMCP_TRANSCRIPT_DIR := $(CURDIR)/target/typed-followups-recovery
typed-followups-recovery:
	cargo test --locked -p biomcp --test cases query-validation/argument-recovery $(TEST_THREADS)
typed-followups-recovery-check:
	+$(MAKE) --keep-going typed-followups-recovery typed-followups-parity check-cheap

.PHONY: enum-case-check
enum-case-check:
	rm -rf "$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-transcripts"
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-transcripts" cargo test --locked -p biomcp --test cases enumerated-letter-case $(TEST_THREADS)
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-transcripts" cargo test --locked -p biomcp --test cases grammar/enum-case.yaml $(TEST_THREADS)

.PHONY: enum-case-prepare enum-case-lint enum-case-schema-check
enum-case-prepare:
	cargo fmt --all
	+$(MAKE) schema-generate docs-generate
	mkdir -p dist
	git diff --binary > dist/enum-case-generated.patch
	+$(MAKE) --keep-going enum-case-check enum-case-lint enum-case-schema-check

enum-case-lint:
	cargo clippy --keep-going --locked -p biomcp -p biomcp-types --all-targets --features biomcp-types/schema -- -D warnings

enum-case-schema-check:
	+$(MAKE) current-schema-check schema-release-check
	cargo test --locked -p biomcp-types --features schema --test schemas enum_field_scope_is_checked_metadata_without_a_schema_major_change

.PHONY: enum-case-fixture-prepare enum-case-fixture-check enum-case-retired-check enum-case-bridge-check
enum-case-fixture-prepare:
	cargo fmt --all
	+$(MAKE) schema-generate docs-generate
	mkdir -p dist
	git diff --binary > dist/enum-case-generated.patch
	+$(MAKE) --keep-going enum-case-fixture-check enum-case-retired-check enum-case-bridge-check enum-case-lint enum-case-schema-check

enum-case-fixture-check:
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-fixture-transcripts" cargo test --locked -p biomcp --test cases grammar/enum-case.yaml $(TEST_THREADS)

enum-case-retired-check:
	cargo test --locked -p biomcp --test cases retired $(TEST_THREADS)

enum-case-bridge-check:
	cargo test --locked -p biomcp --test cases mcp-parity/typed-cli-bridge $(TEST_THREADS)

.PHONY: enum-case-review-fix
enum-case-review-fix:
	cargo fmt --all
	mkdir -p dist
	git diff --binary > dist/enum-case-generated.patch
	+$(MAKE) --keep-going enum-case-check enum-case-lint
.PHONY: population-reply-metadata-check
population-reply-metadata-check:
	+$(MAKE) hosted-surface-generate typed-schema-check
	python3 scripts/trials.py mcp-parity/mcp-typed-tools --case-file variant/population/indel-invalid-crossref.yaml --case-file variant/population/indel-typed.yaml

.PHONY: caid-population-affected caid-population-pre-review
caid-population-affected: caid-population-check
	BIOMCP_TEST_EXECUTABLE="$(CURDIR)/target/debug/biomcp" cargo test --locked -p biomcp --test cases variant/ $(TEST_THREADS)
	BIOMCP_TEST_EXECUTABLE="$(CURDIR)/target/debug/biomcp" cargo test --locked -p biomcp --test cases variant-population/ $(TEST_THREADS)
caid-population-pre-review:
	+$(MAKE) --keep-going caid-population-affected fmt lint-workspace ratchets
.PHONY: trial-provenance-check
trial-provenance-check:
	cargo test --locked -p biomcp --test ratchets required_surface_has_asserted_calls $(TEST_THREADS)
	mkdir -p "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/trial-provenance"
	cargo build --locked -p biomcp --bin biomcp --test cases --message-format=json-render-diagnostics > "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/trial-provenance/build.jsonl"
	python3 -c 'import json,subprocess,sys; rows=[json.loads(line) for line in open(sys.argv[1]) if line.startswith("{")]; executables={row["target"]["name"]:row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("executable")}; subprocess.run([sys.executable,"scripts/trials.py","--runner",executables["cases"],"--executable",executables["biomcp"],"--installed","trial-search/trial/gene-notices.yaml","trial-search/trial/gene-notices-context.yaml","trial-search/trial/gene-notices-mcp-typed-mode"],check=True)' "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/trial-provenance/build.jsonl"

.PHONY: typed-owned-followups-check
typed-owned-followups-check:
	BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/typed-owned-followups" cargo test --locked -p biomcp --test cases mcp-parity/typed-owned- $(TEST_THREADS)

.PHONY: typed-owned-followups-green typed-followups-research
typed-owned-followups-green:
	+$(MAKE) --keep-going typed-owned-followups-check typed-followups-recovery typed-followups-parity typed-followups-toolcases typed-followups-research check-cheap
typed-followups-research:
	cargo test --locked -p biomcp --test cases mcp-research/ $(TEST_THREADS)

.PHONY: typed-grouped-integration-check
typed-grouped-integration-check: export BIOMCP_TRANSCRIPT_DIR := $(CURDIR)/target/typed-grouped-integration
typed-grouped-integration-check:
	+$(MAKE) --keep-going fmt lint-workspace lint-tools
	cargo test --locked -p biomcp --test cases mcp-research/ $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-hosted/served-surface-and-source-policy $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases variant-population/variant/population/indel- $(TEST_THREADS)

.PHONY: typed-followups-hosted-recovery
# Reuse the owner trial without rebuilding an installed release binary.
typed-followups-hosted-recovery: export BIOMCP_TRANSCRIPT_DIR := $(CURDIR)/target/typed-followups-hosted-recovery
typed-followups-hosted-recovery:
	cargo test --locked -p biomcp --test cases mcp-hosted/served-surface-and-source-policy $(TEST_THREADS)
	+$(MAKE) --keep-going typed-followups-recovery typed-followups-parity check-cheap


.PHONY: typed-catalog-followups-composition-check
typed-catalog-followups-composition-check: export BIOMCP_TRANSCRIPT_DIR := $(CURDIR)/target/typed-catalog-followups-composition
typed-catalog-followups-composition-check:
	+$(MAKE) --keep-going check-cheap typed-followups-parity typed-followups-research
	cargo test --locked -p biomcp --test cases mcp-hosted/served-surface-and-source-policy $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases query-validation/argument-recovery $(TEST_THREADS)

.PHONY: enum-case-metadata-prepare enum-case-metadata-check enum-case-front-map-check
enum-case-front-map-check:
	cargo build --locked -p biomcp
	python3 scripts/check_front_map.py "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp"

enum-case-metadata-prepare:
	cargo fmt --all
	+$(MAKE) schema-generate docs-generate
	mkdir -p dist
	git diff --binary > dist/enum-case-generated.patch
	+$(MAKE) --keep-going enum-case-metadata-check enum-case-lint enum-case-schema-check

enum-case-metadata-check:
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-metadata-transcripts" cargo test --locked -p biomcp --test cases mcp-parity/compact-typed-schemas $(TEST_THREADS)
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-metadata-transcripts" cargo test --locked -p biomcp --test cases mcp-parity/typed-hosted-catalog $(TEST_THREADS)
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-metadata-transcripts" cargo test --locked -p biomcp --test cases mcp-parity/typed-help-research-discovery $(TEST_THREADS)
	BIOMCP_TRANSCRIPT_DIR="$(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/enum-case-metadata-transcripts" cargo test --locked -p biomcp --test cases schema-discovery/help-contract $(TEST_THREADS)

.PHONY: gencc-publisher-check
gencc-publisher-check: fmt
	cargo test --locked -p biomcp --test cases gene-sections/gencc-first-publisher $(TEST_THREADS)

.PHONY: accepted-batch-composition-check
accepted-batch-composition-check:
	+$(MAKE) hosted-surface-generate
	+$(MAKE) --keep-going typed-catalog-followups-composition-check enum-case-check article-default-focused
	cargo test --locked -p biomcp --test tools -- --proofs schema-discovery/rejects-bad-cases $(filter-out --,$(TEST_THREADS))
	python3 scripts/measure_typed_catalog.py "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp" "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/accepted-batch-catalog"

.PHONY: accepted-batch-composition-finish accepted-batch-unrun-boundaries accepted-batch-unrun-negative accepted-batch-unrun-profiles
accepted-batch-composition-finish:
	mkdir -p target/accepted-batch-finish
	cargo build --locked -p biomcp --bin biomcp --test cases --test tools --message-format=json-render-diagnostics > target/accepted-batch-finish/build.jsonl
	python3 -c 'import hashlib,json,subprocess; from pathlib import Path; rows=[json.loads(x) for x in Path("target/accepted-batch-finish/build.jsonl").read_text().splitlines() if x.startswith("{")]; artifacts={r["target"]["name"]:{"path":r["executable"],"sha256":hashlib.sha256(Path(r["executable"]).read_bytes()).hexdigest(),"profile":r["profile"]} for r in rows if r.get("reason")=="compiler-artifact" and r.get("executable")}; assert all(x in artifacts for x in ("biomcp","cases","tools")); Path("target/accepted-batch-finish/source-bindings.json").write_text(json.dumps({"revision":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"artifacts":artifacts},indent=2)+"\n")'
	+$(MAKE) --keep-going lint-workspace lint-tools enum-case-check article-default-focused accepted-batch-unrun-boundaries accepted-batch-unrun-negative accepted-batch-unrun-profiles

accepted-batch-unrun-boundaries:
	cargo test --locked -p biomcp --test cases mcp-hosted/served-surface-and-source-policy $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases query-validation/argument-recovery $(TEST_THREADS)

accepted-batch-unrun-negative:
	cargo test --locked -p biomcp --test tools -- --proofs schema-discovery/rejects-bad-cases $(filter-out --,$(TEST_THREADS))

accepted-batch-unrun-profiles:
	python3 scripts/measure_typed_catalog.py "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp" "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/accepted-batch-catalog"

.PHONY: accepted-batch-article-finish
accepted-batch-article-finish:
	cargo clippy --keep-going --locked -p biomcp --test cases --test tools -- -D warnings
	mkdir -p target/article-selector-finish
	BIOMCP_TRANSCRIPT_DIR="$(CURDIR)/target/article-selector-finish" cargo test --locked -p biomcp --test tools -- --cases "$(CURDIR)/tests/tool-cases" article/selector-errors.yaml
	cargo test --locked -p biomcp --test cases grammar/argument-description-guidance $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases grammar/numeric-description-guidance $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases trial/input-boundaries $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases variant/identity $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases article/fulltext/ $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases article/preprint/ $(TEST_THREADS)

.PHONY: hosted-resilience-focused
hosted-resilience-focused:
	cargo test --locked -p biomcp --bin biomcp mcp::http::call::tests $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-hosted/replay-partial-at-deadline $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-http/ $(TEST_THREADS)
	cargo test --locked -p biomcp --test cases mcp-hosted/pacing/whole-call-deadline $(TEST_THREADS)

.PHONY: hosted-resilience-mechanical
hosted-resilience-mechanical:
	+$(MAKE) --no-print-directory check-cheap
	+$(MAKE) --no-print-directory hosted-resilience-focused

.PHONY: commercial-config-check
commercial-config-check:
	python3 scripts/trials.py --case-file readme/gene.yaml --case-file readme/article.yaml --case-file readme/trial.yaml mcp-hosted/use/commercial-config
