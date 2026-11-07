# 0749 — Article search key reads match the sources it calls

Status: COMPLETE.
Landed: d0f18a16c314b5a821a2797057a1f5255db982a8

Milestone: 5

## Outcome

The credential read test passes again and states what article search really calls. Each command reads only the keys of the sources it calls.

## Why

Practice run 42344263032 at c821e374d stopped on Linux Intel at `called_sources_read_only_their_own_environment_key_once`. `search article EGFR --source pubmed` read `OPENFDA_API_KEY` where the test allows only `NCBI_API_KEY`. 0717 made article search look names up in openFDA labels, so the lookup calls openFDA and reads its own key. The landing gate does not run this test, so the failure stayed hidden until the release section ran it.

## Starts from

- Starts from: `crates/biomcp/src/config/credential_reads_tests.rs`, `crates/biomcp-query/src/drug/search_names.rs`.

## Keeps

- Keeps: 0717's drug name lookup, and every other case in the test. A key is read only for a source the command calls.

## Changes

- Changes: the test takes the set of keys each command may read. If the lookup turns out to read a key for a source it does not call, fix the code, not the test.

## Proof

- Proof: the test passes, and it still fails if a command reads a key outside its set.

## Defers

- Defers: nothing.

## Depends on

- Depends on: nothing.

## Evidence

- Evidence: run 42344263032, job 128319065511, step "Check formatting, lint and tests on Linux x86_64": `left: Some("OPENFDA_API_KEY")`, `right: Some("NCBI_API_KEY")`. The other four platforms passed.

## Progress
