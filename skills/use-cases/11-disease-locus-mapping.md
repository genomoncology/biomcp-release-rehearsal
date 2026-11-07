# Find genes in a stated cytoband

## Question

Which source gene records are listed for the cytoband 9p21.3?

## 1. Retrieve and inspect

```sh
biomcp search gene --band 9p21.3
```

Read the echoed cytoband, total, returned gene identities and next page command. The first page is only part of the source listing.

## 2. Retrieve and inspect

```sh
biomcp search gene --band 9p21.3 --limit 50
```

Read the wider page and inspect the listed symbols and HGNC identifiers. Keep all candidates; page order is not a disease ranking.

## Next

Follow all remaining page commands before claiming a complete listing. This maps a cytoband to source gene records; it does not establish a causal gene, a disease association or a base-pair interval.

Read response status, problems and source terms at every step. Live replies can change.
