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

.PHONY: landing-gate plain-cli-build fmt lint ratchets test audit coverage unreached

landing-gate:
	+python3 scripts/landing_gate.py

# Keep these sequential even when the caller uses make -j.
.PHONY: pre-job proof-check proof-list proof proof-unpack gate-stage-check
pre-job:
	+$(MAKE) --no-print-directory fmt
	+$(MAKE) --no-print-directory lint
	+$(MAKE) --no-print-directory ratchets

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

lint:
	+$(MAKE) --no-print-directory unused-dependencies
	cargo check --locked -p biodata -p biomcp-types --no-default-features
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo clippy --locked -p biomcp --test tools -- -D warnings

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
	cargo test --locked -p biomcp --bin biomcp grammar::reference::reference_matches_grammar $(TEST_THREADS)
	python3 release/toolchain.py check
	python3 -m unittest release.test_toolchain
	python3 release/tools.py
	python3 -m unittest release.test_tools release.test_linux release.test_mutations
	python3 scripts/check_docs.py
	python3 scripts/article_licenses.py
	python3 scripts/test_article_licenses.py
	python3 scripts/test_docs.py
	python3 scripts/test_source_capture.py
	python3 scripts/test_source_capture_plan.py
	python3 scripts/test_coverage.py
	python3 scripts/test_reply_diff.py
	python3 scripts/test_schema_release.py
	+$(MAKE) --no-print-directory gate-stage-check
	python3 scripts/test_build_workers.py
	python3 scripts/test_trials.py
	python3 scripts/test_nightly_live_check.py
	python3 -m unittest release.test_installer release.test_checks release.test_gate release.test_windows
	python3 -m unittest release.test_workflow release.test_security release.test_licenses
	python3 -m unittest release.test_source_revision release.test_workflow_contracts
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
