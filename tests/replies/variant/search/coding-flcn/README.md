# Retained ClinVar coding search reply

The public ESearch response was captured on 2026-10-29T21:31:33.638909+00:00. Its unchanged 386 bytes have SHA-256 `79bf1474ae1577db7655f8295f2396714f2084ce12e295f5879f3bf40c7fa758`. The replay stores those bytes as text to preserve their original representation.

The original request was `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=clinvar&term=FLCN%5BGENE%5D+AND+%22c.780-4C%3EG%22%5BVRNM%5D&retmode=json&retmax=50&tool=biomcp`. The replay request uses the accepted canonical URL encoder, which writes spaces as `%20`. It sends no `retstart` for offset zero. The decoded parameters match the original request.

The recording envelope supplies a synthetic zero delay. It does not claim a new provider request or a measured transport duration.
