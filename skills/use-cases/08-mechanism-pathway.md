# Read an identified pathway record

## Question

Which native pathway record matches the stated pathway name?

## 1. Retrieve and inspect

```sh
biomcp search pathway 'Signaling by EGFR'
```

Read candidate identifiers and names. Select an exact source record rather than assuming all name matches are interchangeable.

## 2. Retrieve and inspect

```sh
biomcp get pathway R-HSA-177929
```

Read the native identifier, name, species, version and release. Confirm the source record matches the selected candidate.

## Next

Follow returned source links and section commands to inspect membership. Pathway membership does not establish that a pathway is active in a patient.

Read response status, problems and source terms at every step. Live replies can change.
