.DEFAULT_GOAL := landing-gate

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
.PHONY: pre-job check-cheap proof gate-stage-check
pre-job: check-cheap

LINT_GOALS = unused-dependencies check-no-default-features lint-workspace lint-tools lint-rustdoc

check-cheap:
	python3 scripts/check_docs.py
	+$(MAKE) --no-print-directory --keep-going -j1 shell-check fmt $(LINT_GOALS) ratchets

.PHONY: shell-check
shell-check:
	python3 scripts/check_shell.py

proof: pre-job
	python3 scripts/trials.py --target tools --proofs $(PROOFS)

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
	python3 scripts/check_docs.py --executable "$(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/debug/biomcp"
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

.PHONY: unused-dependencies
unused-dependencies:
	python3 release/toolchain.py install-machete
	cargo machete

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

.PHONY: hosted-surface-check
hosted-surface-check:
	python3 scripts/trials.py --installed mcp-hosted/served-surface-and-source-policy mcp-parity/typed-hosted-catalog
