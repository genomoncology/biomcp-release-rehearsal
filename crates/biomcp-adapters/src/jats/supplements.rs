use biodata::{Markdown, Text};

use biomcp_fetch::BadReply;
/// Explicit JATS supplement declaration before the bucket intersection.
#[derive(Debug)]
pub struct Supplement {
    /// Source filename reference.
    pub href: String,
    /// Explicit source MIME declaration, when present.
    pub kind: Option<biomcp_types::FileKind>,
    /// Optional source label.
    pub label: Option<Text>,
    /// Optional source caption.
    pub caption: Option<Markdown>,
}
pub(super) fn read(
    xml: &str,
    problems: &mut crate::mapped::Problems,
) -> Result<Vec<Supplement>, BadReply> {
    let mut out = Vec::new();
    for item in super::blocks::descendants(xml, "supplementary-material")? {
        let label = label(item, problems)?;
        let caption = caption(item)?;
        let kind = mime(item)?;
        if let Some(href) = item.attr("xlink:href")? {
            out.push(Supplement {
                href,
                kind,
                label: label.clone(),
                caption: caption.clone(),
            });
        }
        for media in super::blocks::descendants(item.0, "media")? {
            if let Some(href) = media.attr("xlink:href")? {
                out.push(Supplement {
                    href,
                    kind: mime(media)?.or(kind),
                    label: label.clone().or(self::label(media, problems)?),
                    caption: caption.clone().or(self::caption(media)?),
                });
            }
        }
    }
    Ok(out)
}
fn label(
    item: crate::xml_element::Element<'_>,
    problems: &mut crate::mapped::Problems,
) -> Result<Option<Text>, BadReply> {
    Ok(item
        .value(&["label"])?
        .filter(|v| !v.trim().is_empty())
        .and_then(|v| problems.optional("supplements.label", Some(Text::from_source(&v)))))
}
fn caption(item: crate::xml_element::Element<'_>) -> Result<Option<Markdown>, BadReply> {
    let Some(caption) = item.child("caption")? else {
        return Ok(None);
    };
    if caption.text()?.trim().is_empty() {
        return Ok(None);
    }
    super::markdown(caption.0).map(Some)
}
fn mime(item: crate::xml_element::Element<'_>) -> Result<Option<biomcp_types::FileKind>, BadReply> {
    use biomcp_types::FileKind;
    let Some(top) = item.attr("mimetype")? else {
        return Ok(None);
    };
    let Some(sub) = item.attr("mime-subtype")? else {
        return Ok(Some(FileKind::Other));
    };
    let kind = match format!("{top}/{sub}").to_ascii_lowercase().as_str() {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => FileKind::Xlsx,
        "text/csv" => FileKind::Csv,
        "text/tab-separated-values" => FileKind::Tsv,
        "text/plain" => FileKind::Txt,
        "application/pdf" => FileKind::Pdf,
        "application/octet-stream" | "binary/octet-stream" => return Ok(None),
        _ => FileKind::Other,
    };
    Ok(Some(kind))
}
