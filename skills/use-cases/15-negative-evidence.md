# Distinguish empty from unavailable source sections

## Question

What does an empty or inapplicable source section actually report?

## 1. Retrieve and inspect

```sh
biomcp get gene MIR21 expression
```

Read the resolved gene and expression section state, source outcomes and problems. An empty section reports no returned rows for this source and request.

## 2. Retrieve and inspect

```sh
biomcp get gene MIR21 pathways
```

Read the independent pathway section state. Inapplicable is distinct from a successful empty search, a disabled source and a failed request.

## Next

Keep each state, query and source with the observation. Do not report an empty or unavailable section as evidence that the gene lacks a biological role or a clinical association.

Read response status, problems and source terms at every step. Live replies can change.
