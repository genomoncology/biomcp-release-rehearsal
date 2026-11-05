#!/bin/sh
set -eu
PATH="$HOME/.local/bin:$PATH"
export PATH
case "${1:-focused}" in
reproduce) exec python3.13 scripts/check_research_fetch.py ;;
focused) exec python3.13 scripts/trials.py mcp-research/fetch-readable-record-once mcp-research/research-search-fetch-parity mcp-research/research-registry-projection-paths mcp-research/research-phenotype-search-fetch --case-file mcp/research-fetch.yaml ;;
*) exit 2 ;;
esac
