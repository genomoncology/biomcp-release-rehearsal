# 0774 — Discover filtered cell-line reply schema

Status: COMPLETE.
Landed: a707db4d568aa335a9f7a2346c10ced24e19a420

Milestone: 5

## Outcome

Complete help/reference metadata and typed MCP output schemas include the actual filtered cell-line reply alongside the literal-search reply.

## Why

The docs generator found that cell-line search emits biomcp.cell-line-filtered-page/1, but schema metadata matches only the Command-trimmed query title.

## Proof

- Proof: a failing then passing complete-help regression and typed MCP output-schema assertion retain both cell-line search reply forms and sibling command mappings.

## Starts from

- Starts from: crates/biomcp/build/schema_metadata.rs and crates/biomcp-query/src/cell_line/mod.rs search_body.

## Keeps

- Keeps: runtime reply shapes, schema identities, and existing output-schema authority.

## Defers

- Defers: nothing.

## Changes

- Changes: generic query-title matching in build metadata and meaningful help/typed-schema regression coverage.
