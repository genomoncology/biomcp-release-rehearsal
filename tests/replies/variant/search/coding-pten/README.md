# Retained ClinVar coding search reply

The public ESearch response was captured on 2026-10-29T21:31:32.500762+00:00. Its unchanged 378 bytes have SHA-256 `f85a6ce3d885931ae986c0af157936c3cd0b44a84d93ec5dcaa77a7425cb6346`. The replay stores those bytes as text to preserve their original representation.

The original request was `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=clinvar&term=PTEN%5BGENE%5D+AND+%22c.333G%3ET%22%5BVRNM%5D&retmode=json&retmax=50&tool=biomcp`. The replay request uses the accepted canonical URL encoder, which writes spaces as `%20`. It sends no `retstart` for offset zero. The decoded parameters match the original request.

The recording envelope supplies a synthetic zero delay. It does not claim a new provider request or a measured transport duration.
