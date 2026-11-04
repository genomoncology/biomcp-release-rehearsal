//! Bounded UTF-8 XML replies and ordered inline text.
use crate::Reply;
use quick_xml::{Reader, events::Event};
use std::borrow::Cow;

// One bound protects ignored XML subtrees as well as serde's recursive visitors.
const NESTING: usize = 64;

#[derive(Clone, Copy)]
pub(crate) enum Codec {
    Json,
    JsonText,
    Xml,
    Bytes(&'static str),
    Probe,
}
impl Codec {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::JsonText => "text/plain",
            Self::Xml => "application/xml",
            Self::Bytes(media) => media,
            Self::Probe => "*/*",
        }
    }
    pub(crate) fn accepts(self, media: &str) -> bool {
        let mut parts = media.split(';');
        let name = parts.next().unwrap_or("").trim().to_ascii_lowercase();
        match self {
            Self::Bytes(_) => true,
            Self::Probe => true,
            Self::Json | Self::JsonText => {
                name == "application/json"
                    || name.starts_with("application/") && name.ends_with("+json")
                    || matches!(self, Self::JsonText) && name == "text/plain"
            }
            Self::Xml => {
                (name == self.name()
                    || name == "text/xml"
                    || name.starts_with("application/") && name.ends_with("+xml"))
                    && parts.all(|part| {
                        let Some((key, value)) = part.split_once('=') else {
                            return true;
                        };
                        !key.trim().eq_ignore_ascii_case("charset")
                            || value.trim().trim_matches('"').eq_ignore_ascii_case("utf-8")
                    })
            }
        }
    }
}

/// Strict XML media and UTF-8 charset admission for source policies.
pub fn accepts_media(media: &str) -> bool {
    Codec::Xml.accepts(media)
}

/// An XML reply selects its raw decoding at the document boundary.
/// Ordinary fields use serde by default. Mixed elements use `MixedText::from_xml`.
pub trait XmlReply: Reply {
    /// Admit source media while ordinary XML keeps strict UTF-8 admission.
    fn accepts_media(media: &str) -> bool {
        accepts_media(media)
    }
    /// Convert source encoding before shared XML validation and cache admission.
    fn normalize_body<'a>(_media: &str, body: &'a [u8]) -> Cow<'a, [u8]> {
        Cow::Borrowed(body)
    }

    /// Decode raw XML without exposing parser messages to the caller.
    fn decode_xml(xml: &str) -> Result<Self, crate::BadReply> {
        let mut de = quick_xml::de::Deserializer::from_str(xml);
        de.recursion_limit(NESTING);
        Self::deserialize(&mut de).map_err(|_| crate::BadReply)
    }
}

pub(crate) fn validate(text: &str) -> Option<()> {
    let mut reader = Reader::from_str(text);
    reader.config_mut().enable_all_checks(true);
    let mut depth = 0;
    let mut roots = 0;
    let mut declaration = false;
    let mut doctype = false;
    loop {
        let event = reader.read_event().ok()?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(tag) | Event::Empty(tag) => {
                // Empty events consume one level too, but do not retain it.
                if depth == 0 {
                    roots += 1;
                }
                if roots > 1 || depth >= NESTING {
                    return None;
                }
                for attribute in tag.attributes() {
                    attribute
                        .ok()?
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .ok()?;
                }
                if !empty {
                    depth += 1;
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1)?;
            }
            Event::Eof => return (roots == 1 && depth == 0).then_some(()),
            Event::Decl(decl) => {
                if declaration
                    || roots != 0
                    || doctype
                    || reader.buffer_position() as usize != decl.len() + 4
                {
                    return None;
                }
                declaration = true;
                validate_declaration(&decl)?;
            }
            Event::DocType(_) => {
                if roots != 0 || doctype {
                    return None;
                }
                doctype = true;
            }
            Event::Text(value)
                if depth == 0
                    && !value
                        .as_ref()
                        .chars()
                        .all(|c| matches!(c, ' ' | '\n' | '\r' | '\t')) =>
            {
                return None;
            }
            Event::GeneralRef(value) => {
                if depth == 0 {
                    return None;
                }
                if value.resolve_char_ref().ok()?.is_none()
                    && !matches!(value.as_ref(), "lt" | "gt" | "amp" | "apos" | "quot")
                {
                    return None;
                }
            }
            Event::CData(_) if depth == 0 => return None,
            _ => {}
        }
    }
}

fn validate_declaration(decl: &quick_xml::events::BytesDecl<'_>) -> Option<()> {
    decl.xml_version().ok()?;
    let attributes = quick_xml::events::BytesStart::from_content(decl.as_ref(), 3);
    let mut previous = 0;
    for attribute in attributes.attributes() {
        let attribute = attribute.ok()?;
        let order = match attribute.key.as_ref() {
            "version" => 1,
            "encoding" if attribute.value.eq_ignore_ascii_case("utf-8") => 2,
            "standalone" if matches!(attribute.value.as_ref(), "yes" | "no") => 3,
            _ => return None,
        };
        if order <= previous {
            return None;
        }
        previous = order;
    }
    Some(())
}

/// All text within an XML element, with inline markup removed in document order.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct MixedText(String);
impl MixedText {
    /// Borrow text without changing whitespace or inserting separators.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Read one complete element, including arbitrary nested inline text.
    /// Attributes and comments contribute no text. DTDs are never resolved.
    pub fn from_xml(element: &str) -> Result<Self, crate::BadReply> {
        let element = element.strip_prefix('\u{feff}').unwrap_or(element);
        validate(element).ok_or(crate::BadReply)?;
        let mut reader = Reader::from_str(element);
        let mut text = String::new();
        let mut depth = 0;
        loop {
            match reader.read_event().map_err(|_| crate::BadReply)? {
                Event::Start(_) => depth += 1,
                Event::End(_) => depth -= 1,
                Event::Text(value) if depth != 0 => text.push_str(value.as_ref()),
                Event::CData(value) => text.push_str(value.as_ref()),
                Event::GeneralRef(value) => {
                    if let Some(ch) = value.resolve_char_ref().map_err(|_| crate::BadReply)? {
                        text.push(ch);
                    } else {
                        text.push_str(
                            quick_xml::escape::resolve_predefined_entity(value.as_ref())
                                .ok_or(crate::BadReply)?,
                        );
                    }
                }
                Event::Eof => return Ok(Self(text)),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "xml_tests.rs"]
mod tests;

#[derive(serde::Deserialize)]
pub(crate) struct Probe;
impl Reply for Probe {
    fn check(&self) -> Result<(), crate::BadReply> {
        Ok(())
    }
}
#[derive(Debug, serde::Deserialize)]
/// Original bytes returned by a checked binary read.
pub struct Bytes(pub Vec<u8>);
impl Reply for Bytes {
    fn check(&self) -> Result<(), crate::BadReply> {
        Ok(())
    }
    fn check_status(&self, status: u16) -> Result<(), crate::BadReply> {
        // Binary callers retrieve complete resources and never send Range requests.
        if status == 200 {
            Ok(())
        } else {
            Err(crate::BadReply)
        }
    }
}
