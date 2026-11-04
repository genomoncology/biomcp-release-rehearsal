use super::{blocks, markdown, parts::Part};
use crate::xml_element::Element;
use biodata::Text;
use biomcp_fetch::BadReply;
use biomcp_types::TextPartKind;
/// One source row and its rendered representation, before bounded fragmentation.
#[derive(Debug)]
pub struct BodyRow {
    /// Original paragraphs owned by this source row, including nested list paragraphs.
    pub paragraphs: Vec<super::references::Paragraph>,
    /// Source text used for literal matching.
    pub plain: String,
    /// Whole rendered source row.
    pub markdown: String,
    /// This row supplies a table's caption or column headings.
    pub context: bool,
    /// Explicit citation reference identities for this source row.
    pub references: Box<[Text]>,
}
#[derive(Default)]
pub(super) struct Rows {
    pub(super) values: Vec<BodyRow>,
    pub(super) bytes: usize,
}
pub(super) struct Group {
    pub(super) title: Option<String>,
    pub(super) path: Vec<Text>,
    pub(super) kind: TextPartKind,
    pub(super) context: Option<String>,
    pub(super) rows: Vec<BodyRow>,
    pub(super) table_refs: Vec<Text>,
}
impl Group {
    pub(super) fn separator(&self) -> &'static str {
        if self.kind == TextPartKind::Table {
            "\n"
        } else {
            "\n\n"
        }
    }
    pub(super) fn section(path: Vec<Text>) -> Self {
        Self {
            title: Some(path.last().map_or("Body", Text::as_str).to_owned()),
            path,
            kind: TextPartKind::Section,
            context: None,
            rows: Vec::new(),
            table_refs: Vec::new(),
        }
    }
}
pub(super) fn section(
    node: Element<'_>,
    ancestors: &[Text],
    problems: &mut crate::mapped::Problems,
) -> Result<Vec<Group>, BadReply> {
    let mut path = ancestors.to_vec();
    let title = node.value(&["title"])?;
    let checked_title = title
        .as_ref()
        .and_then(|title| problems.text("parts.title", Some(title)));
    let invalid_title = title.is_some() && checked_title.is_none();
    if let Some(title) = checked_title {
        path.push(title);
    }
    let mut group = Group::section(path.clone());
    if invalid_title {
        group.title = None;
    }
    if let Some(title) = node.child("title")?.filter(|_| !invalid_title) {
        group.rows.push(BodyRow {
            paragraphs: Vec::new(),
            plain: title.text()?,
            markdown: format!(
                "{} {}",
                "#".repeat(path.len() + 1),
                markdown::inline(title.0)?.trim()
            ),
            context: false,
            references: super::references::ids(title.0)?,
        });
    }
    let mut out = Vec::new();
    for (name, child) in blocks::body_nodes(node.0)? {
        if !matches!(name.as_str(), "sec" | "table-wrap" | "table-wrap-group") {
            group
                .table_refs
                .extend(super::references::table_ids(child.0)?);
        }
        match name.as_str() {
            "title" | "table-wrap" | "table-wrap-group" => {}
            "sec" => {
                if !group.rows.is_empty() {
                    out.push(group);
                }
                out.extend(section(child, &path, problems)?);
                group = Group::section(path.clone());
                if invalid_title {
                    group.title = None;
                }
            }
            "fig" => {
                if let Some(row) = figure(child)? {
                    group.rows.push(row);
                }
            }
            _ => {
                let plain = child.text()?;
                if !plain.trim().is_empty() {
                    group.rows.push(BodyRow {
                        paragraphs: super::references::paragraphs(child.0)?,
                        plain,
                        markdown: markdown::inline(child.0)?.trim().to_owned(),
                        context: false,
                        references: super::references::ids(child.0)?,
                    });
                }
            }
        }
    }
    if !group.rows.is_empty() {
        out.push(group);
    }
    Ok(out)
}
pub(super) fn figure(fig: Element<'_>) -> Result<Option<BodyRow>, BadReply> {
    let Some(caption) = fig.child("caption")? else {
        return Ok(None);
    };
    let label = fig.value(&["label"])?.unwrap_or_else(|| "Figure".into());
    let plain = caption.text()?;
    if plain.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(BodyRow {
        paragraphs: super::references::paragraphs(caption.0)?,
        plain,
        markdown: format!(
            "**{}.** {}",
            label.trim_end_matches('.'),
            markdown::inline(caption.0)?.trim()
        ),
        context: false,
        references: super::references::ids(caption.0)?,
    }))
}
pub(super) fn append(
    parts: &mut Vec<Part>,
    group: Group,
    rows: &mut Rows,
    problems: &mut crate::mapped::Problems,
) -> Result<(), BadReply> {
    super::parts::append(parts, group, rows, problems)
}

pub(super) fn groups(
    name: &str,
    node: Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<Vec<Group>, BadReply> {
    if name == "sec" {
        return section(node, &[], problems);
    }
    let mut group = Group::section(Vec::new());
    group.table_refs = super::references::table_ids(node.0)?.into_vec();
    let rendered = if name == "fig" {
        figure(node)?
    } else {
        let plain = node.text()?;
        if plain.trim().is_empty() {
            None
        } else {
            Some(BodyRow {
                paragraphs: super::references::paragraphs(node.0)?,
                plain,
                markdown: markdown::inline(node.0)?.trim().to_owned(),
                context: false,
                references: super::references::ids(node.0)?,
            })
        }
    };
    if let Some(row) = rendered {
        group.rows.push(row);
    }
    Ok(vec![group])
}
