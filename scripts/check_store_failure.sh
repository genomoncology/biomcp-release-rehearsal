#!/bin/sh
set -eu
case "${1:-focused}" in
red) cargo test --locked -p biomcp --test cases article-search/late-store-lock-keeps-fetched-items ;;
cheap) make fmt lint ratchets ;;
schema) make schema-generate ;;
article) cargo test --locked -p biomcp-query --test article_search ;;
focused)
 cargo test --locked -p biomcp --test cases article-search/current-store-opens-without-write-lock
 cargo test --locked -p biomcp --test cases late-store-lock-keeps-fetched
 cargo test --locked -p biomcp --test cases cache-resilience/checked
 sh "$0" retention
 ;;
retention)
 cargo fmt --all -- --check
 cargo clippy --locked -p biomcp --test cases -- -D warnings
 cargo test --locked -p biomcp --test cases cache-resilience/stale-replies
 cargo test --locked -p biomcp --test cases held-store-lock
 cargo test --locked -p biomcp --test cases store-clean/
 ;;
*) exit 2 ;;
esac
