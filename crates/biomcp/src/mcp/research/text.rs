//! A literal readable rendering keeps source markup from becoming instructions or links.
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use serde_json::Value;
use std::io;

pub(super) fn render(value: &Value) -> io::Result<String> {
    let mut value = value.clone();
    clean(&mut value);
    let markdown = String::from_utf8(crate::view::markdown(&value)?).map_err(io::Error::other)?;
    let decoded = entities(&markdown);
    let mut text = String::new();
    let mut chars = decoded.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek().is_some_and(|c| "\\*_`|[]#!~".contains(*c)) {
            text.push(chars.next().unwrap_or_default());
        } else {
            text.push(ch);
        }
    }
    // A fence longer than any source or generated fence keeps the entire view inert.
    let width = text.split(|ch| ch != '`').map(str::len).max().unwrap_or(0) + 1;
    let fence = "`".repeat(width.max(3));
    Ok(format!("{fence}text\n{}\n{fence}\n", text.trim_end()))
}

fn clean(value: &mut Value) {
    match value {
        Value::String(text) => *text = source(text),
        Value::Array(items) => items.iter_mut().for_each(clean),
        Value::Object(fields) => fields.values_mut().for_each(clean),
        _ => {}
    }
}

pub(super) fn source(text: &str) -> String {
    let decoded = entities(text);
    let mut out = String::new();
    let mut remaining = decoded.as_str();
    let mut opened = Vec::<String>::new();
    while let Some(start) = remaining.find('<') {
        let (prefix, tail) = remaining.split_at(start);
        out.push_str(prefix);
        remaining = tail;
        let name = remaining
            .strip_prefix("</")
            .or_else(|| remaining.strip_prefix('<'))
            .unwrap_or(remaining);
        if name.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '!') {
            let mut reader = Reader::from_str(remaining);
            reader.config_mut().check_end_names = false;
            reader.config_mut().allow_unmatched_ends = true;
            let marker = match reader.read_event() {
                Ok(Event::Start(tag))
                    if markup(&tag)
                        && (void(tag.name().as_ref())
                            || has_end(remaining, tag.name().as_ref())) =>
                {
                    let name = tag.name().as_ref().to_ascii_lowercase();
                    if !void(&name) {
                        opened.push(name.clone());
                    }
                    Some(match name.as_str() {
                        "sup" => "^(",
                        "sub" => "_(",
                        "br" => "\n",
                        _ => "",
                    })
                }
                Ok(Event::Empty(tag)) if markup(&tag) => {
                    Some(if tag.name().as_ref().eq_ignore_ascii_case("br") {
                        "\n"
                    } else {
                        ""
                    })
                }
                Ok(Event::End(tag))
                    if opened
                        .last()
                        .is_some_and(|name| tag.name().as_ref().eq_ignore_ascii_case(name)) =>
                {
                    Some(match opened.pop().as_deref() {
                        Some("sup" | "sub") => ")",
                        _ => "",
                    })
                }
                Ok(Event::Comment(comment)) => {
                    out.push_str(comment.as_ref());
                    Some("")
                }
                _ => None,
            };
            if let Some(marker) = marker {
                out.push_str(marker);
                remaining = remaining.split_at(reader.buffer_position() as usize).1;
                continue;
            }
        }
        out.push('<');
        remaining = remaining.split_at(1).1;
    }
    out.push_str(remaining);
    out
}

// Only recognized markup with valid attributes may discard angle-delimited bytes.
// Scientific comparisons and unknown source tokens remain literal inside the fence.
fn markup(tag: &BytesStart<'_>) -> bool {
    markup_name(tag.name().as_ref()) && tag.attributes().all(|attribute| attribute.is_ok())
}

fn void(name: &str) -> bool {
    ["br", "hr"]
        .iter()
        .any(|tag| name.eq_ignore_ascii_case(tag))
}

fn has_end(text: &str, name: &str) -> bool {
    text.match_indices("</").any(|(start, _)| {
        let mut reader = Reader::from_str(&text[start..]);
        reader.config_mut().allow_unmatched_ends = true;
        matches!(reader.read_event(), Ok(Event::End(tag)) if tag.name().as_ref().eq_ignore_ascii_case(name))
    })
}

fn markup_name(name: &str) -> bool {
    [
        "a",
        "b",
        "blockquote",
        "bold",
        "br",
        "code",
        "dd",
        "div",
        "dl",
        "dt",
        "em",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "hr",
        "i",
        "italic",
        "li",
        "monospace",
        "ol",
        "p",
        "pre",
        "s",
        "small",
        "span",
        "strike",
        "strong",
        "sub",
        "sup",
        "table",
        "tbody",
        "td",
        "th",
        "thead",
        "tr",
        "tt",
        "u",
        "ul",
        "underline",
    ]
    .iter()
    .any(|tag| name.eq_ignore_ascii_case(tag))
}

fn entities(text: &str) -> String {
    let mut out = String::new();
    let mut remaining = text;
    while let Some(start) = remaining.find('&') {
        let (prefix, tail) = remaining.split_at(start);
        out.push_str(prefix);
        remaining = tail;
        if let Some(end) = remaining
            .as_bytes()
            .iter()
            .take(64)
            .position(|ch| *ch == b';')
        {
            let (reference, tail) = remaining.split_at(end + 1);
            if let Ok(decoded) = quick_xml::escape::unescape(reference) {
                out.push_str(&decoded);
                remaining = tail;
                continue;
            }
        }
        out.push('&');
        remaining = remaining.split_at(1).1;
    }
    out.push_str(remaining);
    out
}
