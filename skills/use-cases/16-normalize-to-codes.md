# Keep normalization services independent

## Question

Which representations does a named normalization service return for an explicit HGVS input?

## 1. Retrieve and inspect

```sh
biomcp variant normalize --source mutalyzer --hgvs 'NM_004333.4:c.1799T>A'
```

Read the exact input accession and version, source result, messages and returned mappings. Preserve any uncertainty or refusal.

## 2. Retrieve and inspect

```sh
biomcp variant normalize --source variantvalidator --hgvs 'NM_004333.4:c.1799T>A' --build GRCh38
```

Read this service’s independent result and explicit requested assembly. Compare identifiers and mappings without selecting a consensus locally.

## Next

Use a returned explicit identifier for subsequent retrieval after checking it matches the intended allele. Normalization does not classify pathogenicity or select a clinically appropriate transcript.

Read response status, problems and source terms at every step. Live replies can change.
