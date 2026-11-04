# Country recordings

The one-country registry search retains the complete unmodified response values from a bounded ClinicalTrials.gov request on 2026-10-29. The source returned NCT03734029 with United States locations. The original response body SHA-256 is `dfad43b49141fb6b0e7daae2d526422bb57573ddfe748d9032c0a814089ba8eb`. The exact native request uses `AREA[NCTId]NCT03734029 AND (AREA[LocationCountry]"United States")` and requests LocationCountry alongside the existing header fields.

The US/Canada and Afghanistan recordings are explicit request-mapping stubs that reuse that same unmodified native response. The Afghanistan stub exercises refusal of a source response that contradicts the requested country. Their request identities are local test inputs and do not claim original upstream captures. The NCI recording is another explicit request-mapping stub. It reuses the existing native facility-search response to check repeated `sites.org_country` parameters and query echo. It makes no claim about the countries in that NCI response.

The source-version response was captured separately on 2026-10-29 under the same bounded recipe. The complete NCT03734029 get recording comes unchanged from the existing trial recordings. The get recording independently verifies the first returned trial's US site. These public registry values support discovery and make no eligibility claim.
