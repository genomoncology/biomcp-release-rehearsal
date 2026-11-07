# 0746 — Skill map shows commands as typed

Status: OPEN.

Milestone: 5

## Outcome

The skill's command map shows commands in the order they are typed, such as `search article`, so an agent reading it makes no `article search` or `article get` calls.

## Why

An outside agent test read the map's `article: ...get,search` entry as entity first. `article get` failed in MCP and took the script's other calls with it, and `article search` failed in the CLI. Fixing the text is simpler than accepting two orders, and it adds no new command forms.

## Starts from

- Starts from: the skill's map, and any generated text that repeats its form.

## Keeps

- Keeps: command grammar and refusals.

## Changes

- Changes: the skill text, its generator if it has one, and its tests.

## Proof

- Proof: every command in the map runs through the built CLI's parser, in a test that fails on the old form.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.
