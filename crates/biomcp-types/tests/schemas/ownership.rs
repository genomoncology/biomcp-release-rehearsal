//! Check public schema namespaces against the crate that defines each Rust type.
use super::registry;

#[test]
fn schema_ids_follow_type_ownership() {
    for contract in registry() {
        let namespace = if contract.rust_type.starts_with("biodata::") {
            "biodata."
        } else {
            assert!(contract.rust_type.starts_with("biomcp_types::"));
            "biomcp."
        };
        assert!(
            contract.id.starts_with(namespace) && !contract.id.contains('_'),
            "{} ({}) must use {namespace} and hyphens: {}",
            contract.name,
            contract.rust_type,
            contract.id
        );
    }
}
