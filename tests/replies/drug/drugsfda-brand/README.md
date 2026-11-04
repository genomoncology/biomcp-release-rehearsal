# Drugs@FDA brand discovery

The successful response body was captured unchanged from the public openFDA Drugs@FDA endpoint on 2026-10-30. The native request was `https://api.fda.gov/drug/drugsfda.json?search=products.brand_name.exact%3A%22SARCLISA+ESCENA%22&limit=1`. It returned HTTP 200 and 3,356 bytes. The original SHA-256 is `b37c8cdbd2c8b0f1bacc1fc218f531255ca5244cc1836a00f9de1240d09a3a2a`.

The recording wrappers exercise the broader brand-or-ingredient request, ingredient search, and exact application retrieval against that same unchanged native application. Those wrappers are local request fixtures, rather than separate provider captures. They preserve the native single-application count and limit. The legacy brand-as-ingredient request has a synthetic absence reply so the old behavior fails on the missing application. The legacy ingredient wrapper also permits an unchanged-reply comparison against the published executable.

The case replaces the synthetic `literal-application-candidates` runner trial. It checks the native application identity and products, the get handoff, and the independent submission status date. The brand and ingredient names do not identify a chosen product or establish therapeutic equivalence.
