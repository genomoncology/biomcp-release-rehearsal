# 0764 — Docs and skill name the entity tools

Status: COMPLETE.
Landed: 1f8f2165fc8b341d402717ae6c49e1ee3779b99a

Milestone: 5

## Outcome

README, docs, the plugin skill, `configuration.md` and `release/notes.md` describe the 17 entity tools from 0763, with one example per tool. Nothing public names `biomcp_reference` or a retired tool. The release notes tell 0.x and hosted users which old tool maps to which new tool and action.

## Why

0763 changes every typed tool name, and hosted users and the plugin read these docs (`decisions/2026-11-07-entity-tools-in-1-0.md`).

## Proof

- Proof: the docs check fails on any retired typed tool name outside the release notes' mapping table. Every example call in the docs passes the same validation that 0758 added for tool descriptions.

## Starts from

- Starts from: the files that name `biomcp_reference`, `biomcp_help` or a `get_*`/`search_*` typed tool outside `crates/` and `sdlc/`, and the existing docs check script.

## Keeps

- Keeps: CLI docs and the generated command reference, which 0763 does not change.

## Defers

- Defers: nothing.

## Changes

- Changes: README, `docs/`, the plugin skill, `configuration.md`, `release/notes.md`, the docs check.

## Depends on

- Depends on: 0763.
