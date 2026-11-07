# Inspect native variant assertions

## Question

Which assertions does a named source return for this explicit variant record?

## 1. Retrieve and inspect

```sh
biomcp variant assertions clinvar:13961 --source clinvar
```

Read the resolved allele identity, submitted assertion, condition, submitter, review information and dates where supplied. Preserve differences between assertions.

## 2. Retrieve and inspect

```sh
biomcp get variant clinvar:13961 clinvar
```

Inspect the same source section through the get route. Confirm the requested identity and source outcomes before following record links.

## Next

Review the underlying submissions and evidence. Do not turn an aggregate label into a patient diagnosis or a treatment recommendation.

Read response status, problems and source terms at every step. Live replies can change.
