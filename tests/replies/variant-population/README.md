These fixtures contain synthetic response-only edits over request identities emitted by the actual CLI in job j10866 at 0b45d47d. GWAS source paths, request methods, canonical filenames, URLs and JSON request body hashes are unchanged. gnomAD request identities were updated through the actual recorder as described below. The original replies, CLI output, executable digest and capture manifests are retained outside Git because redistribution rights in raw source replies remain unverified.

The BRAF control preserves the reviewed public GRCh38 source allele, dataset, exome allele count and denominator, and null genome. Its per-group counts and filter are explicitly synthetic; group IDs use the documented source grammar. The recorded reply does not establish a dataset release, so native counts remain available while typed frequency projection is withheld. The GWAS zero reproduces the reviewed public source-local zero and preserves the current v2 rsID request. Error status, timing and malformed bodies are synthetic; three status exchanges exercise the existing bounded retry policy. No fixture supplies a clinical classification or a derived allele frequency.

The gnomAD request wrappers use the actual CLI recorder identity emitted by the local synthetic TLS case in job j11213 at afd0c3be. The expanded request asks for VA release, count and cohort fields in that same response. Existing exchanges, including bodies and original fetch dates, remain unchanged. Their missing VA releases still yield native counts and a dataset-version problem. These are reused synthetic responses delivered locally, not new provider captures. The live release-bearing allele capture remains separate.

`source-ca-identity.json` minimizes the existing byte-faithful MyVariant capture
`myvariant/get_braf_v600e_grch38_20260806.json` from the source fixture archive.
The original receipt gives SHA-256
`78f7159bf545e417740ddf3781643c267b18cef2e21da9e7c34aa5e2830f1aed`
and request
`https://myvariant.info/v1/variant/chr7%3Ag.140753336A%3ET?assembly=hg38`.
Only the source record identity, coordinate metadata, rsID and ClinGen CAid remain.
The query envelope and downstream count reply in the local transport case are synthetic.

0610 extends the same gnomAD request with source `homozygote_count` totals and nullable `joint { ac an homozygote_count }`. The four prior variant-request wrappers are rebound to the new query body and canonical POST identity; their exchanges and numeric fields are unchanged. Missing fields stay absent rather than becoming zero.

`homozygotes-joint` contains actual public gnomAD, MyVariant and CAR responses captured on 2026-11-04. Its original gnomAD request/response bytes and response SHA-256 are retained beside the replay wrapper in `source-binding.json`. The witness publishes exome homozygotes 80805, genome homozygotes 6918, joint AC 513548, AN 1613846 and homozygotes 87723. Matching VA ALL rows publish 4.1.0 for exome and genome; the summary names that shared version, not a separate joint release. No values are summed.

`no-joint` and `conflicting-release` are explicitly synthetic response-only controls over that unchanged witness numeric data. Only joint is replaced by null in the first; only the genome ALL release label becomes `synthetic-conflicting-release` in the second. They are not claims of live absence or a published conflicting release. The actual BRAF source query returned joint 2/1612922/0 and is not presented as a no-joint witness.

Publisher schemas: [variant totals](https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/variant.graphql) and [VA provenance](https://github.com/broadinstitute/gnomad-browser/blob/main/graphql-api/src/graphql/types/va.graphql). VA has exome and genome release metadata but no independent joint branch.

`joint-without-genome` keeps the actual BRAF gnomAD/CAR responses: genome is null, but joint AN is 1612922 rather than the exome AN 1460618. The source reports zero homozygotes. This control would fail if joint totals were constructed from available subset counts, and distinguishes a published zero from absent joint data. Original gnomAD bytes and their SHA-256 are retained beside the request binding.

Product joint data uses the existing section states: data for supplied totals, empty for null or absent source joint data, and unavailable for a refused joint record. Actual gnomAD exome/genome filters are explicitly `[]` in both live captures. The runner preserves that existing typed GnomadCounts contract only at its exact filters paths; unrelated empty lists and product nulls remain rejected.
