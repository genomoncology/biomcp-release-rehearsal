use super::{
    blocks,
    body::{BodyRow, Group},
    markdown, references,
};
use crate::xml_element::Element;
use biodata::Text;
use biomcp_fetch::BadReply;
use biomcp_types::TextPartKind;
const MAX_COLUMNS: usize = 256;
struct Cells {
    expanded: Vec<String>,
    plain: String,
    references: Box<[Text]>,
    paragraphs: Vec<super::references::Paragraph>,
}
pub(super) fn group(table: Element<'_>, path: &[Text]) -> Result<Group, BadReply> {
    let label = table.value(&["label"])?.unwrap_or_else(|| "Table".into());
    let caption = table.child("caption")?;
    let text = caption
        .map(|v| markdown::inline(v.0))
        .transpose()?
        .unwrap_or_default();
    let caption = format!("**{}.** {}", label.trim_end_matches('.'), text.trim());
    let mut source = Vec::new();
    let mut expanded = 0usize;
    for row in blocks::descendants(table.0, "tr")? {
        let cells = cells(row)?;
        expanded = cells
            .expanded
            .iter()
            .try_fold(expanded, |n, v| n.checked_add(v.len()).ok_or(BadReply))?;
        if expanded > super::MAX_MARKDOWN_BYTES {
            return Err(BadReply);
        }
        if !cells.expanded.is_empty() {
            source.push(cells);
        }
    }
    let width = source.iter().map(|r| r.expanded.len()).max().unwrap_or(0);
    let mut rendered = Vec::new();
    for cells in source {
        let mut md: Vec<_> = cells
            .expanded
            .into_iter()
            .map(|m| m.replace('|', "\\|").replace('\n', "<br>"))
            .collect();
        md.resize(width, String::new());
        rendered.push(BodyRow {
            paragraphs: cells.paragraphs,
            plain: cells.plain,
            markdown: format!("| {} |", md.join(" | ")),
            context: false,
            references: cells.references,
        });
    }
    let mut rows = vec![BodyRow {
        paragraphs: table
            .child("caption")?
            .map(|node| references::paragraphs(node.0))
            .transpose()?
            .unwrap_or_default(),
        plain: format!("{label} {}", table.value(&["caption"])?.unwrap_or_default()),
        markdown: caption.clone(),
        context: true,
        references: references::ids(table.0)?,
    }];
    let mut context = caption;
    if !rendered.is_empty() {
        let mut header = rendered.remove(0);
        header.context = true;
        let separator = format!("| {} |", vec!["---"; width].join(" | "));
        context.push_str(&format!("\n\n{}\n{separator}", header.markdown));
        rows.push(header);
        rows.push(BodyRow {
            paragraphs: Vec::new(),
            plain: String::new(),
            markdown: separator,
            context: true,
            references: Box::default(),
        });
    }
    rows.extend(rendered);
    Ok(Group {
        title: Some(label),
        path: path.to_vec(),
        kind: TextPartKind::Table,
        context: Some(context),
        rows,
        table_refs: Vec::new(),
    })
}
fn cells(row: Element<'_>) -> Result<Cells, BadReply> {
    let mut out = Cells {
        expanded: Vec::new(),
        plain: String::new(),
        references: references::ids(row.0)?,
        paragraphs: references::paragraphs(row.0)?,
    };
    let mut bytes = 0usize;
    for (name, cell) in blocks::body_nodes(row.0)? {
        if !matches!(name.as_str(), "td" | "th") {
            continue;
        }
        let span = cell
            .attr("colspan")?
            .map(|v| v.parse::<usize>().map_err(|_| BadReply))
            .transpose()?
            .unwrap_or(1);
        if span == 0 || span > MAX_COLUMNS || out.expanded.len() + span > MAX_COLUMNS {
            return Err(BadReply);
        }
        let value = markdown::inline(cell.0)?.trim().to_owned();
        let expanded = value.len().checked_mul(span).ok_or(BadReply)?;
        bytes = bytes
            .checked_add(expanded)
            .filter(|n| *n <= super::MAX_MARKDOWN_BYTES)
            .ok_or(BadReply)?;
        if !out.plain.is_empty() {
            out.plain.push(' ');
        }
        out.plain.push_str(&cell.text()?);
        out.expanded.extend(std::iter::repeat_n(value, span));
    }
    Ok(out)
}
