//! Add schema-owned aliases for the public GWAS command.
use serde_json::Value;
pub(super) fn aliases(command: clap::Command, schema: &Value) -> clap::Command {
    if schema.pointer("/properties/query").is_some() {
        command
            .mut_arg("gene", |arg| arg.short('g'))
            .mut_arg("limit", |arg| arg.short('l'))
    } else {
        command
    }
}
