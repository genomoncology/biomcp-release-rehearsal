# Inspect protein features after gene resolution

## Question

Which protein record and annotated features can I inspect for EGFR?

## 1. Retrieve and inspect

```sh
biomcp get protein EGFR
```

Inspect the gene match and returned protein accession. If the reply lists candidates, resolve the identity before selecting a protein.

## 2. Retrieve and inspect

```sh
biomcp protein features P00533
```

Read source feature types, positions and qualifiers against the exact accession. Retain source coordinate conventions and any sequence version.

## 3. Retrieve and inspect

```sh
biomcp protein domains P00533
```

Read each source domain identifier and interval. Keep domain records distinct from a statement about a particular variant’s effect.

## Next

Follow returned protein section and page commands. An annotation’s position does not establish a clinical consequence.

Read response status, problems and source terms at every step. Live replies can change.
