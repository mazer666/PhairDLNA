//! A bounded reader for the XML a control point sends.
//!
//! What: turns a bounded SOAP document into a namespace-resolved element tree.
//! Why: hostile network input needs parser-backed syntax checks and explicit limits.
//! Example: `parse(r#"<s:Envelope xmlns:s="urn:soap"/>"#)` resolves the prefix.

use quick_xml::events::Event;
use quick_xml::reader::Reader;

use crate::upnp::error::UpnpError;

use super::xml_syntax::{
    declarations, decode_entities, decode_entity, is_declaration, resolve, split_name,
};

/// The bounds a parsed document must stay inside.
#[derive(Debug, Clone, Copy)]
pub struct XmlLimits {
    /// The largest document that is read, in bytes.
    pub max_bytes: usize,
    /// The deepest nesting that is read; the root element has depth zero.
    pub max_depth: usize,
    /// The largest number of elements in the document.
    pub max_nodes: usize,
    /// The largest text of a single element, in bytes.
    pub max_text_bytes: usize,
}

impl Default for XmlLimits {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024,
            max_depth: 32,
            max_nodes: 1_000,
            max_text_bytes: 32 * 1024,
        }
    }
}

/// One element of a parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    /// The local name of the element, without its prefix.
    pub name: String,
    /// The namespace the element belongs to, or an empty string.
    pub namespace: String,
    /// The attributes of the element, without the namespace declarations.
    pub attributes: Vec<(String, String)>,
    /// The child elements.
    pub children: Vec<Element>,
    /// The text directly inside the element.
    pub text: String,
}

impl Element {
    /// The child element with the given name, if the element has one.
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|child| child.name == name)
    }

    /// The text of the child element with the given name.
    pub fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|child| child.text.as_str())
    }
}

/// Reads a document with the default bounds.
pub fn parse(input: &str) -> Result<Element, UpnpError> {
    parse_with_limits(input, &XmlLimits::default())
}

/// Reads a document with the given bounds.
pub fn parse_with_limits(input: &str, limits: &XmlLimits) -> Result<Element, UpnpError> {
    if input.len() > limits.max_bytes {
        return Err(UpnpError::RequestTooLarge {
            limit: limits.max_bytes,
        });
    }
    parse_bounded_document(input, limits)
}

fn parse_bounded_document(input: &str, limits: &XmlLimits) -> Result<Element, UpnpError> {
    let mut reader = Reader::from_str(input);
    reader.config_mut().check_comments = true;
    let mut parser = TreeParser::new(limits);
    loop {
        let event = reader.read_event().map_err(|_| malformed_xml())?;
        if parser.accept(event)? {
            return parser.finish();
        }
    }
}

struct TreeParser<'a> {
    limits: &'a XmlLimits,
    nodes: usize,
    elements: Vec<Element>,
    scopes: Vec<Vec<(String, String)>>,
    root: Option<Element>,
}

impl<'a> TreeParser<'a> {
    fn new(limits: &'a XmlLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            elements: Vec::new(),
            scopes: vec![vec![(
                "xml".to_owned(),
                "http://www.w3.org/XML/1998/namespace".to_owned(),
            )]],
            root: None,
        }
    }

    fn accept(&mut self, event: Event<'_>) -> Result<bool, UpnpError> {
        match event {
            Event::Start(start) => self.start_element(&start)?,
            Event::Empty(start) => self.empty_element(&start)?,
            Event::End(_) => self.end_element()?,
            Event::Text(text) => self.text(&text)?,
            Event::CData(text) => self.cdata(&text)?,
            Event::GeneralRef(reference) => self.reference(&reference)?,
            Event::DocType(_) => return Err(malformed_xml()),
            Event::Eof => return Ok(true),
            Event::Decl(_) | Event::PI(_) | Event::Comment(_) => {}
        }
        Ok(false)
    }

    fn start_element(
        &mut self,
        start: &quick_xml::events::BytesStart<'_>,
    ) -> Result<(), UpnpError> {
        self.count_node()?;
        let (element, scope) = self.build_element(start)?;
        self.elements.push(element);
        self.scopes.push(scope);
        Ok(())
    }

    fn empty_element(
        &mut self,
        start: &quick_xml::events::BytesStart<'_>,
    ) -> Result<(), UpnpError> {
        self.count_node()?;
        let (element, _) = self.build_element(start)?;
        self.attach(element)
    }

    fn count_node(&mut self) -> Result<(), UpnpError> {
        if self.elements.len() > self.limits.max_depth {
            return Err(UpnpError::RequestTooDeep {
                limit: self.limits.max_depth,
            });
        }
        self.nodes += 1;
        if self.nodes > self.limits.max_nodes {
            return Err(UpnpError::TooManyNodes {
                limit: self.limits.max_nodes,
            });
        }
        Ok(())
    }

    fn build_element(
        &self,
        start: &quick_xml::events::BytesStart<'_>,
    ) -> Result<(Element, Vec<(String, String)>), UpnpError> {
        let raw_name = start.name();
        let raw_name = raw_name.as_ref();
        let attributes = read_attributes(start)?;
        let mut scope = self.scopes.last().cloned().unwrap_or_default();
        scope.extend(declarations(&attributes));
        let (prefix, name) = split_name(raw_name);
        let namespace = resolve(&scope, prefix)?;
        let attributes = attributes
            .into_iter()
            .filter(|(name, _)| !is_declaration(name))
            .collect();
        Ok((
            Element {
                name,
                namespace,
                attributes,
                children: Vec::new(),
                text: String::new(),
            },
            scope,
        ))
    }

    fn end_element(&mut self) -> Result<(), UpnpError> {
        let element = self.elements.pop().ok_or_else(malformed_xml)?;
        self.scopes.pop();
        self.attach(element)
    }

    fn attach(&mut self, element: Element) -> Result<(), UpnpError> {
        if let Some(parent) = self.elements.last_mut() {
            parent.children.push(element);
        } else if self.root.replace(element).is_some() {
            return Err(malformed_xml());
        }
        Ok(())
    }

    fn text(&mut self, text: &quick_xml::events::BytesText<'_>) -> Result<(), UpnpError> {
        let decoded = decode_entities(text.as_ref())?;
        if let Some(element) = self.elements.last_mut() {
            element.text.push_str(&decoded);
            let length = element.text.len();
            check_text_length(self.limits, length)
        } else if decoded.trim().is_empty() {
            Ok(())
        } else {
            Err(malformed_xml())
        }
    }

    fn cdata(&mut self, text: &quick_xml::events::BytesCData<'_>) -> Result<(), UpnpError> {
        let element = self.elements.last_mut().ok_or_else(malformed_xml)?;
        element.text.push_str(text.as_ref());
        let length = element.text.len();
        check_text_length(self.limits, length)
    }

    fn reference(&mut self, reference: &quick_xml::events::BytesRef<'_>) -> Result<(), UpnpError> {
        let decoded = decode_entity(reference.as_ref())?;
        let element = self.elements.last_mut().ok_or_else(malformed_xml)?;
        element.text.push_str(&decoded);
        let length = element.text.len();
        check_text_length(self.limits, length)
    }

    fn finish(self) -> Result<Element, UpnpError> {
        if !self.elements.is_empty() {
            return Err(malformed_xml());
        }
        self.root.ok_or_else(malformed_xml)
    }
}

fn check_text_length(limits: &XmlLimits, length: usize) -> Result<(), UpnpError> {
    if length > limits.max_text_bytes {
        return Err(UpnpError::TextTooLong {
            limit: limits.max_text_bytes,
        });
    }
    Ok(())
}

fn read_attributes(
    start: &quick_xml::events::BytesStart<'_>,
) -> Result<Vec<(String, String)>, UpnpError> {
    start
        .attributes()
        .with_checks(true)
        .map(|attribute| {
            let attribute = attribute.map_err(|_| malformed_xml())?;
            let name = attribute.key.as_ref().to_owned();
            let raw_value = attribute.value.as_ref();
            Ok((name, decode_entities(raw_value)?))
        })
        .collect()
}

fn malformed_xml() -> UpnpError {
    UpnpError::MalformedRequest {
        reason: "the XML document is malformed",
    }
}
