//! GPML's bounded XML document and native DataNode fields.
use biomcp_fetch::{BadReply, Reply, xml::XmlReply};
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use serde::Deserialize;

pub(super) const MAX_BYTES: u64 = 8 * 1024 * 1024;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 64;
const NAMESPACE: &str = "http://pathvisio.org/GPML/2013a";

#[derive(Debug, Deserialize)]
pub(super) struct Document {
    pub version: String,
    pub organism: String,
    pub name: Option<String>,
    pub nodes: Vec<Node>,
}
#[derive(Debug, Deserialize)]
pub(super) struct Node {
    pub kind: Option<String>,
    pub id: Option<String>,
    pub label: Option<String>,
    pub namespace: Option<String>,
    pub value: Option<String>,
    #[serde(skip)]
    xref_seen: bool,
}
fn attribute(tag: &BytesStart<'_>, key: &str) -> Result<Option<String>, BadReply> {
    let mut found = None;
    for attribute in tag.attributes() {
        let attribute = attribute.map_err(|_| BadReply)?;
        if attribute.key.as_ref() == key {
            found = Some(
                attribute
                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                    .map_err(|_| BadReply)?
                    .into_owned(),
            );
        }
    }
    Ok(found)
}
fn required(tag: &BytesStart<'_>, key: &str) -> Result<String, BadReply> {
    attribute(tag, key)?
        .filter(|s| !s.is_empty())
        .ok_or(BadReply)
}
impl Document {
    fn document_header(tag: &BytesStart<'_>) -> Result<Self, BadReply> {
        if tag.name().as_ref() != "Pathway"
            || required(tag, "xmlns")? != NAMESPACE
            || required(tag, "Data-Source")? != "WikiPathways"
        {
            return Err(BadReply);
        }
        Ok(Self {
            version: required(tag, "Version")?,
            organism: required(tag, "Organism")?,
            name: attribute(tag, "Name")?,
            nodes: Vec::new(),
        })
    }
    pub(super) fn matches(&self, id: &biodata::WikiPathwaysId) -> Result<(), BadReply> {
        let revision = self
            .version
            .strip_prefix(&format!("{id}_r"))
            .ok_or(BadReply)?;
        if self.organism != "Homo sapiens"
            || revision.parse::<u64>().ok().filter(|n| *n > 0).is_none()
            || !revision.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(BadReply);
        }
        Ok(())
    }
}
impl Reply for Document {
    fn check(&self) -> Result<(), BadReply> {
        if self.nodes.len() > MAX_NODES {
            return Err(BadReply);
        }
        Ok(())
    }
}
impl XmlReply for Document {
    fn decode_xml(xml: &str) -> Result<Self, BadReply> {
        if xml.len() as u64 > MAX_BYTES {
            return Err(BadReply);
        }
        let mut reader = Reader::from_str(xml);
        reader.config_mut().enable_all_checks(true);
        let (mut depth, mut count) = (0usize, 0usize);
        let (mut document, mut node) = (None::<Document>, None::<Node>);
        loop {
            let event = reader.read_event().map_err(|_| BadReply)?;
            let empty = matches!(event, Event::Empty(_));
            match event {
                Event::Start(tag) | Event::Empty(tag) => {
                    count += 1;
                    if count > MAX_NODES || depth >= MAX_DEPTH {
                        return Err(BadReply);
                    }
                    if depth == 0 {
                        if document.is_some() {
                            return Err(BadReply);
                        }
                        document = Some(Self::document_header(&tag)?);
                    } else {
                        read_child(&tag, depth, &mut node)?;
                    }
                    if empty {
                        finish_node(&tag, depth, &mut document, &mut node)?;
                    } else {
                        depth += 1;
                    }
                }
                Event::End(tag) => {
                    depth = depth.checked_sub(1).ok_or(BadReply)?;
                    if depth == 1 && tag.name().as_ref() == "DataNode" {
                        push_node(&mut document, &mut node)?;
                    }
                }
                Event::DocType(_) => return Err(BadReply),
                Event::Eof => return document.filter(|_| depth == 0).ok_or(BadReply),
                _ => {}
            }
        }
    }
}
fn read_child(tag: &BytesStart<'_>, depth: usize, node: &mut Option<Node>) -> Result<(), BadReply> {
    if ((depth == 1 && tag.name().as_ref() == "DataNode")
        || (depth == 2 && node.is_some() && tag.name().as_ref() == "Xref"))
        && attribute(tag, "xmlns")?.is_some_and(|namespace| namespace != NAMESPACE)
    {
        return Err(BadReply);
    }
    if depth == 1 && tag.name().as_ref() == "DataNode" {
        *node = Some(Node {
            kind: attribute(tag, "Type")?,
            id: attribute(tag, "GraphId")?,
            label: attribute(tag, "TextLabel")?,
            namespace: None,
            value: None,
            xref_seen: false,
        });
    } else if depth == 2
        && tag.name().as_ref() == "Xref"
        && let Some(node) = node
    {
        if node.xref_seen {
            return Err(BadReply);
        }
        node.xref_seen = true;
        node.namespace = attribute(tag, "Database")?;
        node.value = attribute(tag, "ID")?;
    }
    Ok(())
}
fn finish_node(
    tag: &BytesStart<'_>,
    depth: usize,
    document: &mut Option<Document>,
    node: &mut Option<Node>,
) -> Result<(), BadReply> {
    if depth == 1 && tag.name().as_ref() == "DataNode" {
        push_node(document, node)?;
    }
    Ok(())
}
fn push_node(document: &mut Option<Document>, node: &mut Option<Node>) -> Result<(), BadReply> {
    let node = node.take().ok_or(BadReply)?;
    document.as_mut().ok_or(BadReply)?.nodes.push(node);
    Ok(())
}
