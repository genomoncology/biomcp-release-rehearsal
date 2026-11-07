# Find published evidence for a treatment question

## Question

Which articles match the stated KRAS G12C and sotorasib question?

## 1. Retrieve and inspect

```sh
biomcp search article 'KRAS G12C sotorasib' --since 2024 --sort date --limit 5
```

Read the echoed query, publication dates, titles and source outcomes. The date selector can match either electronic or print dates. Keep the returned publication identifiers.

## 2. Retrieve and inspect

```sh
biomcp search article 'KRAS G12C sotorasib' --since 2024 --sort date --limit 5 --offset 5
```

Read the next page of the same dated query. Keep the original filters, inspect the returned publication identifiers and continue through returned page commands. Open a relevant returned article before drawing a conclusion.

## Next

Use each response’s returned article commands and page commands. A search match does not establish a treatment indication, benefit or patient fit.

Read response status, problems and source terms at every step. Live replies can change.
