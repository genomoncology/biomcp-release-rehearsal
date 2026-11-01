#!/bin/sh
set -eu
export RUSTUP_TOOLCHAIN=1.95.0
case "${1:-static}" in
  static)
    make fmt
    cargo clippy --workspace --all-targets --locked -- -D warnings
    make ratchets
    ;;
  focused)
    cargo test --locked -p biomcp-types --features schema --test query --test diagnostic_contracts --test disease_contracts --test drug_contracts --test phenotype_contracts
    cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types
    cargo test --locked -p biomcp --test cases disease/
    cargo test --locked -p biomcp --test cases diagnostic/
    cargo test --locked -p biomcp --test cases drug/
    cargo test --locked -p biomcp --test cases variant/
    cargo test --locked -p biomcp --test cases phenotype/
    ;;
  review-fixes)
    sh "$0" static
    cargo test --locked -p biomcp-types --features schema --test variant_values typed_candidate_assertions_preserve_source_identity_and_page
    cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types
    cargo test --locked -p biomcp --test cases diagnostic/pma.yaml
    cargo test --locked -p biomcp --test cases variant/
    ;;
  drug-final)
    sh "$0" static
    sh "$0" drug
    ;;
  drug)
    cargo test --locked -p biomcp-types --features schema --test drug_contracts
    cargo test --locked -p biomcp-types --features schema --test schemas checked_in_schemas_match_types
    cargo test --locked -p biomcp --test cases drug/
    ;;
  *) exit 2 ;;
esac
