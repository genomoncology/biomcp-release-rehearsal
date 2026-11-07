# Resolve an ambiguous gene name before disease review

## Question

How do I avoid silently selecting one gene from an ambiguous alias?

## 1. Retrieve and inspect

```sh
biomcp get gene p16
```

Read the ambiguous resolution and all candidate HGNC identifiers. Preserve the source match kind and candidate names.

## 2. Retrieve and inspect

```sh
biomcp search gene p16 --band 9p21.3
```

This example adds an explicitly stated cytoband to the alias search. Inspect the echoed selector, candidate identifiers and match kinds. Use the band only when it is part of the question; a narrower result does not supply missing clinical context.

## Next

Choose a candidate only from the question’s stated identity or independently reviewed evidence. A gene alias and its candidate list cannot disambiguate a syndrome or diagnose a patient.

Read response status, problems and source terms at every step. Live replies can change.
