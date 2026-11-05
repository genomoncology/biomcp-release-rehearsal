# 0665 — Use the canonical AlphaGenome HTTPS endpoint

Status: COMPLETE.
Landed: 5018220c7ef76fca8286591f19f7b3c60f26ce37

Milestone: 5

## Outcome

AlphaGenome uses its canonical HTTPS endpoint without a redundant default port, and its compiled public endpoint and neighboring diagnostic text pass artifact screening while real credential findings still fail.

## Why

The superseded release's staged artifact scan found one benign compiled-text adjacency in twelve package representations; a fresh reviewer traced every finding to these two public source literals, with no credential involved.

## Proof

- Proof: reproduce the joined public literals from current main through the existing artifact scanner, show the canonical endpoint removes that finding, retain a positive credential fixture, and run the existing AlphaGenome admission and local TLS/proxy checks plus the routine gate. Before the next cut, the staged-artifact section passes on its real host under 0640.

## Starts from

- Starts from: `crates/biomcp-adapters/src/alphagenome/mod.rs`, `crates/biomcp-adapters/src/strict_input.rs`, `crates/biomcp-adapters/tests/alphagenome_registration.rs`, and `crates/biomcp-fetch/src/grpc/connect.rs`. Both direct and proxy connections already default HTTPS to port 443. The original stage log and twelve safe finding coordinates are retained under `data/biomcp-redux-evidence/0293/tooling-R-c9bd901d50a5/terminal-superseded/` on Yellow.

## Keeps

- Keeps: the provider destination, TLS and proxy behavior, required-key admission, terms, quota, scanner rules, artifact coverage and real-secret rejection. Never print credential values or weaken the scanner to accept the artifact.

## Defers

- Defers: nothing.

## Changes

- Changes: `crates/biomcp-adapters/src/alphagenome/mod.rs`, `crates/biomcp-adapters/tests/alphagenome_registration.rs`, `release/test_security.py`; the canonical endpoint and focused regression.
