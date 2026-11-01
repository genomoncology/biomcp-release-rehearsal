# Worked commands

Generated from recorded public cases. Run `make docs-generate` to update. The documentation check executes these cases through CLI and MCP. Live replies can differ.

## Which BioMCP version is installed?

```sh
biomcp version
```

## Which source record identifies KRAS?

```sh
biomcp get gene KRAS
```

## Which PubMed articles match AKT1 E17K?

```sh
biomcp search article 'AKT1 E17K'
```

## Which registered trials match melanoma?

```sh
biomcp search trial --condition melanoma
```

