//! Borrowed XML element search within transport-checked documents.
use biomcp_fetch::{BadReply, xml::MixedText};
use quick_xml::{Reader, events::Event};
/// Borrowed element in the transport-checked document.
#[derive(Clone, Copy)]
pub(crate) struct Element<'a>(pub(crate) &'a str);
impl<'a> Element<'a> {
    pub(crate) fn root(xml: &'a str, wanted: &str) -> Result<Self, BadReply> {
        let mut reader = Reader::from_str(xml);
        loop {
            let begin = reader.buffer_position() as usize;
            match reader.read_event().map_err(|_| BadReply)? {
                Event::Start(tag) | Event::Empty(tag) => {
                    return if tag.name().as_ref() == wanted {
                        Ok(Self(&xml[begin..]))
                    } else {
                        Err(BadReply)
                    };
                }
                Event::Eof => return Err(BadReply),
                _ => {}
            }
        }
    }

    pub(crate) fn children(self, wanted: &str) -> Result<Vec<Self>, BadReply> {
        Ok(body_nodes(self.0)?
            .into_iter()
            .filter(|(name, _)| name == wanted)
            .map(|(_, node)| node)
            .collect())
    }
    pub(crate) fn child(self, name: &str) -> Result<Option<Self>, BadReply> {
        Ok(self.children(name)?.into_iter().next())
    }
    pub(crate) fn at(self, path: &[&str]) -> Result<Option<Self>, BadReply> {
        let mut node = Some(self);
        for part in path {
            node = match node {
                Some(n) => n.child(part)?,
                None => None,
            };
        }
        Ok(node)
    }
    pub(crate) fn text(self) -> Result<String, BadReply> {
        Ok(MixedText::from_xml(self.0)?.as_str().to_owned())
    }
    pub(crate) fn value(self, path: &[&str]) -> Result<Option<String>, BadReply> {
        self.at(path)?.map(Self::text).transpose()
    }
    pub(crate) fn attr(self, name: &str) -> Result<Option<String>, BadReply> {
        let mut reader = Reader::from_str(self.0);
        match reader.read_event().map_err(|_| BadReply)? {
            Event::Start(tag) | Event::Empty(tag) => {
                for attr in tag.attributes() {
                    let attr = attr.map_err(|_| BadReply)?;
                    if attr.key.as_ref() == name {
                        return Ok(Some(
                            attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                .map_err(|_| BadReply)?
                                .into_owned(),
                        ));
                    }
                }
                Ok(None)
            }
            _ => Err(BadReply),
        }
    }
}

/// Find all non-nested matching elements in source order.
pub(crate) fn descendants<'a>(xml: &'a str, name: &str) -> Result<Vec<Element<'a>>, BadReply> {
    let mut reader = Reader::from_str(xml);
    let mut out = Vec::new();
    loop {
        let begin = reader.buffer_position() as usize;
        match reader.read_event().map_err(|_| BadReply)? {
            Event::Start(tag) if tag.name().as_ref() == name => {
                reader.read_to_end(tag.name()).map_err(|_| BadReply)?;
                out.push(Element(&xml[begin..reader.buffer_position() as usize]));
            }
            Event::Empty(tag) if tag.name().as_ref() == name => {
                out.push(Element(&xml[begin..reader.buffer_position() as usize]));
            }
            Event::Eof => return Ok(out),
            _ => {}
        }
    }
}

/// Preserve direct elements in document order, including unnamed content.
pub(crate) fn body_nodes(xml: &str) -> Result<Vec<(String, Element<'_>)>, BadReply> {
    let mut reader = Reader::from_str(xml);
    let mut root = false;
    let mut out = Vec::new();
    loop {
        let begin = reader.buffer_position() as usize;
        match reader.read_event().map_err(|_| BadReply)? {
            Event::Start(_) if !root => root = true,
            Event::Start(tag) => {
                let name = tag.name().as_ref().to_owned();
                reader.read_to_end(tag.name()).map_err(|_| BadReply)?;
                out.push((
                    name,
                    Element(&xml[begin..reader.buffer_position() as usize]),
                ));
            }
            Event::Empty(tag) if root => {
                out.push((
                    tag.name().as_ref().to_owned(),
                    Element(&xml[begin..reader.buffer_position() as usize]),
                ));
            }
            Event::Eof => return Ok(out),
            _ => {}
        }
    }
}
