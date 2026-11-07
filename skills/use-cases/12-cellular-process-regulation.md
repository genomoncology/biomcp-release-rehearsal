# Inspect pathway membership and events

## Question

What genes and events does the source list within an exact human pathway?

## 1. Retrieve and inspect

```sh
biomcp pathway genes R-HSA-5673001
```

Read the pathway identity, native gene identifiers and pagination. Retain the source’s membership scope.

## 2. Retrieve and inspect

```sh
biomcp pathway genes R-HSA-5673001 details
```

Read expanded source fields for the same pathway and preserve each gene’s identity.

## 3. Retrieve and inspect

```sh
biomcp pathway events R-HSA-5673001
```

Read event identities and source relationships separately from gene membership.

## Next

Follow returned pages and source event links. Membership and event listings do not measure regulation or patient-specific pathway activity. BioMCP does not run pathway analysis tools.

Read response status, problems and source terms at every step. Live replies can change.
