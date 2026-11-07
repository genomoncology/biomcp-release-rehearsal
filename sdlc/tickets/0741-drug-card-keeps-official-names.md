# 0741 — Drug card keeps official names

Status: OPEN.

Milestone: 5

## Outcome

Official drug names, from the source's INN, USAN, BAN, JAN and FDA synonym types, are kept on the drug card. The drug name cap in search always keeps them, so `acetaminophen` also searches `paracetamol`.

## Why

0717 keeps the record's preferred name, but ChEMBL's preferred name for this drug is ACETAMINOPHEN and `paracetamol` is one synonym among 99 trade names. The drug card keeps no synonym types, so the code cannot tell them apart. A search under the US name misses papers and trials titled with the international one.

## Starts from

- Starts from: the ChEMBL mapping, the drug card, and `crates/biomcp-query/src/drug/search_names.rs`.

## Keeps

- Keeps: 0717's name set not depending on which name was typed, and the cap. No store format, cache namespace or BioData pin change.

## Changes

- Changes: `crates/biomcp-adapters/`, `crates/biomcp-query/src/drug/`, the regenerated drug card schema, and `tests/cases/`.

## Proof

- Proof: `search trial` and `search article` through the built CLI give one total for `acetaminophen` and `paracetamol`, and a case with a real ChEMBL reply shape shows an official name kept past the cap.

## Defers

- Defers: nothing.

## Depends on

- Depends on: 0717.
