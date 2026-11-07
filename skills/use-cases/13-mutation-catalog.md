# Inspect candidates at an explicit genomic position

## Question

Which source variant records overlap this stated region and assembly?

## 1. Retrieve and inspect

```sh
biomcp search variant --region chr7:55259515-55259515 --build GRCh37 --limit 2
```

Read the echoed one-based inclusive region and assembly, source query and every candidate representation. Different alternate alleles can share an rsID.

## Next

Follow returned pages and explicit variant record commands. Preserve assembly and reference-qualified expressions. Candidates do not rank pathogenicity.

Read response status, problems and source terms at every step. Live replies can change.
