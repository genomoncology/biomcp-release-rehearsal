# Read an exact regulatory medicine record

## Question

What regulatory fields belong to the exact EMA record I requested?

## 1. Retrieve and inspect

```sh
biomcp get drug EMEA/H/C/000406 --source ema
```

Inspect the exact application identifier, product name and available sections. A source record identifier is distinct from a molecular identity.

## 2. Retrieve and inspect

```sh
biomcp get drug EMEA/H/C/000406 regulatory --source ema
```

Read jurisdiction, source status and dates with their named roles. Keep separate applications separate.

## Next

If the stored bundle is missing, use the explicit sync command returned in the reply and repeat the read. Follow the regulator’s record for current product information; this record does not choose treatment.

Read response status, problems and source terms at every step. Live replies can change.
