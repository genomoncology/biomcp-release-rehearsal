# Source transcript replay

`myvariant-hsd17b4.json` contains the exact response body from one public MyVariant annotation GET on 2026-10-27. Its SHA-256 is `e881312674468ed0d961fad7c9e12131287d2912c1144b2fee11fa5ea9250138`.

The request was `https://myvariant.info/v1/variant/chr5%3Ag.118860951A%3EG?fields=_id%2Cdbsnp.rsid%2Cdbsnp.gene.symbol%2Cclinvar.variant_id%2Csnpeff.ann&assembly=hg19`. It returned HTTP 200. The response retains the publisher's component license links.

The replay keeps each source transcript identifier, coding notation and protein notation together in source order. The search replies wrap that captured record in synthetic one-hit pages. They prove product mapping and matching through the CLI and MCP. They do not claim a current live search result or supply absent source assembly metadata. The coding-match TLS test uses a synthetic resolver identity and source transcript rows from this record.

`myvariant-clinvar-577152.json` retains the exact MyVariant query response from 2026-10-30. Its SHA-256 is `8801b813527f34848f5c06adb70cc7e9f8aa59d28fce98c0a141f526074eda77`.

The request was `https://myvariant.info/v1/query?q=clinvar.variant_id%3A577152&size=1000&from=0&fields=_id%2Cdbsnp.rsid%2Cdbsnp.gene.symbol%2Cclinvar.variant_id%2Csnpeff.ann&assembly=hg19`. The response supplies the ClinVar spelling replay and retains publisher component license links. It supplies no ATM transcript mapping. The ATM request spelling proof uses a synthetic empty resolver response and makes no allele identity claim.
