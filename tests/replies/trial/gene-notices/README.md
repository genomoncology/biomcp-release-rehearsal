# Gene-word context

The two `.body` files retain the complete ClinicalTrials.gov search responses for `--condition "pancreatic cancer" --gene KRAS --phase 3 --status RECRUITING`, captured on 2026-11-05. Both return 11 total studies and the first ten in the same order. The source requests and decoded responses are also retained in the transport recordings.

- Original header projection: `search-current-main.body`, SHA256 `9e112923a35b97f31953f05a0ae476d06ef6033678e092c22656089d03b83f08`, 7,380 bytes.
- Additional eligibility, description and keyword projection: `search-gene-context.body`, SHA256 `81f264cbdaec1333fbcdc0548c3df86f23a41b7940d51b50a4aa0612810dcf71`, 47,813 bytes.

The cited trial's actual title says “Without KRAS Mutations,” and its inclusion text says “KRAS wild type.” The cases do not relabel this as an exclusion-heading-only mention or infer eligibility. The returned neuroendocrine trial has no exact KRAS mention in these returned protocol fields.

The seven BRAF protocol rows, cursor and count are synthetic. They distinguish inclusion, exclusion-only and mixed wording, whole-word boundaries, unrelated “with or without” wording, negative context, pagination and count-only requests. The HGNC fixture is an explicitly synthetic two-row subset of an existing captured registry input. The version response reuses an older capture; it establishes no new source processing date.
