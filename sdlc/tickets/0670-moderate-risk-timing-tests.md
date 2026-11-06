# 0670 — Moderate-risk timing tests start their budgets after setup

Status: OPEN.

Milestone: 5

Depends on: 0641

## Outcome

The eleven moderate-risk tests in `records/2026-11-05-timing-tests-under-load.md` (rows 5 to 15: fetch retries, cache freshness, transport, gRPC worker, bundle download, HTTP overload and the HGNC snapshot wait) follow the test rule from 0669. Each starts its budget after the setup it is not testing, ends it at a point the test controls through the existing stub holds, or accepts earlier expiry, and keeps the assertion it exists for.

## Why

These share the shape that broke two gates (0667) but have no recorded failure yet. The widened 0641 load run orders them by what it finds.

## Proof

- Proof: the widened 0641 load run passes for every touched test, and each test still fails on the defect it guards. Routine gate.

## Starts from

- Starts from: the record's moderate-risk table with file and line for each test, and the fixture holds already in the gRPC and HTTP stubs.

## Keeps

- Keeps: production code and deadlines. No retries, flaky marks or wider budgets as the fix.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-fetch/tests/`, `crates/biomcp/tests/runner/http_limits.rs`, `crates/biomcp/tests/runner/hgnc_sync.rs`. Test code only.

## Amendment, 2026-11-06

From the ticket review, `records/2026-11-06-ticket-review.md`:

The fixes need only the table, so they can start before 0641. Only the Proof waits for 0641's goal. Line numbers have moved: `retries.rs:145`, `freshness.rs:7,98`, `transport.rs:365,395`, `grpc_worker.rs:317,427,508`, `download.rs:168` and `hgnc_sync.rs:29`. `scripts/deadline_setup_load.py` is not reused as it stands, because it names a fixed remote host.
