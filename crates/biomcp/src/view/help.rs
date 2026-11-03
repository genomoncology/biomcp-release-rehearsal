//! Command help uses headings and copyable shell examples.

use serde_json::Value;

pub(super) fn render(value: &Value, out: &mut String) {
    for command in value
        .pointer("/result/commands")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if command.get("command").is_none() {
            out.push_str(command["description"].as_str().unwrap_or_default());
            out.push_str("\n\n");
            continue;
        }
        let path = command["command"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        if command.get("arguments").is_none()
            && command.get("sections").is_none()
            && command.get("examples").is_none()
            && command.get("reply_schemas").is_none()
        {
            out.push_str(&format!(
                "- biomcp {}: {}\n",
                escape(&path),
                escape(command["description"].as_str().unwrap_or_default())
            ));
            continue;
        }
        out.push_str(&format!(
            "# biomcp {}\n\n{}\n\n",
            escape(&path),
            command["description"]
                .as_str()
                .unwrap_or_default()
                .lines()
                .map(escape)
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
        if let Some(schemas) = command["reply_schemas"]
            .as_array()
            .filter(|v| !v.is_empty())
        {
            out.push_str("## Reply schemas\n\n");
            out.push_str(
                &schemas
                    .iter()
                    .filter_map(Value::as_str)
                    .map(escape)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push_str("\n\n");
        }
        if let Some(arguments) = command["arguments"].as_array().filter(|a| !a.is_empty()) {
            out.push_str("## Arguments\n\n");
            table(arguments, "name", "Name", out);
        }
        if let Some(rule) = command["section_rule"].as_str() {
            out.push_str(&format!("{}\n\n", escape(rule)));
        }
        if let Some(sections) = command["sections"].as_array().filter(|a| !a.is_empty()) {
            out.push_str("## Sections\n\n");
            table(sections, "word", "Word", out);
        }
        for example in command["examples"].as_array().into_iter().flatten() {
            let args = example
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str);
            if let Ok(line) = shlex::try_join(args) {
                let line = super::controls(&line);
                out.push_str(&format!("```sh\nbiomcp {line}\n```\n\n"));
            }
        }
    }
}

fn escape(text: &str) -> String {
    super::escape(text)
        .replace("&#58;", ":")
        .replace("&#46;", ".")
        .replace("&#64;", "@")
}
fn table(rows: &[Value], key: &str, title: &str, out: &mut String) {
    out.push_str(&format!("| {title} | Description |\n| --- | --- |\n"));
    for row in rows {
        let choices = row["choices"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(escape)
            .collect::<Vec<_>>()
            .join(", ");
        let description = escape(row["description"].as_str().unwrap_or_default());
        let description = if choices.is_empty() {
            description
        } else {
            format!("{description} Values: {choices}")
        };
        out.push_str(&format!(
            "| {} | {} |\n",
            escape(row[key].as_str().unwrap_or_default()),
            description
        ));
    }
    out.push('\n');
}
