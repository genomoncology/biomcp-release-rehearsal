//! Explain source policy from the same registry used by health.
use biomcp_query::sources as registry;
use biomcp_types::{Help, HelpCommand};

pub(super) fn help(topic: &str) -> Help {
    let services = registry::sources()
        .map(|source| source.source.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let components = registry::components()
        .map(|component| format!("{}:{}", component.source, component.name))
        .collect::<Vec<_>>()
        .join(", ");
    let mut description = String::from(
        "Every source and carried field is enabled by default. Publisher terms remain authoritative. \
        Commercial users must set {\"disable_sources\":[\"noncommercial\",\"licensed\"]}. \
        Use config disable_sources or --disable-sources with source names or groups; exclusions combine. \
        Groups use the registry's current terms categories. Group consequences follow. ",
    );
    description.insert_str(
        0,
        &format!("Services: {services}. Components: {components}. "),
    );
    for group in registry::disable_groups() {
        let sources = group
            .sources
            .iter()
            .map(|&source| format!("{} ({source})", registry::display_name(source)))
            .collect::<Vec<_>>()
            .join(", ");
        let components = group
            .components
            .iter()
            .map(|c| format!("{}:{}", c.source, c.name))
            .collect::<Vec<_>>()
            .join(", ");
        description.push_str(&format!(
            "{} turns off sources: {sources}. Carried fields: {components}. ",
            group.group.as_str()
        ));
    }
    description.push_str("Carried-field exclusions leave the aggregator's other fields available. PMC Open Access (pmc-oa) supplies PMC reading.");
    Help {
        commands: vec![HelpCommand {
            reply_schemas: crate::schema::local_replies(&["help".into()]),
            command: vec!["help".into(), topic.into()],
            description,
            arguments: Vec::new(),
            sections: Vec::new(),
            section_rule: None,
            examples: Vec::new(),
        }],
    }
}
