use crate::Result;
use pest::{iterators::Pairs, Parser};

use crate::error::Error;
use crate::grammar::Grammar;
use crate::Rule;

pub mod element;
pub mod formatting;
pub mod node;

use element::{Element, ElementVariant};
use node::Node;

/// Document, DocumentFragment or Empty
#[derive(Debug, Clone, PartialEq)]
pub enum DomVariant {
    /// This means that the parsed html had the representation of an html document. The doctype is optional but a document should only have one root node with the name of html.
    /// Example:
    /// ```text
    /// <!doctype html>
    /// <html>
    ///     <head></head>
    ///     <body>
    ///         <h1>Hello world</h1>
    ///     </body>
    /// </html>
    /// ```
    Document,
    /// A document fragment means that the parsed html did not have the representation of a document. A fragment can have multiple root children of any name except html, body or head.
    /// Example:
    /// ```text
    /// <h1>Hello world</h1>
    /// ```
    DocumentFragment,
    /// An empty dom means that the input was empty
    Empty,
}

/// **The main struct** & the result of the parsed html
#[derive(Debug, Clone, PartialEq)]
pub struct Dom<'input> {
    /// The type of the tree that was parsed
    pub tree_type: DomVariant,

    /// All of the root children in the tree
    pub children: Vec<Node<'input>>,

    /// A collection of all errors during parsing
    pub errors: Vec<String>,
}

impl<'input> Default for Dom<'input> {
    fn default() -> Self {
        Self {
            tree_type: DomVariant::Empty,
            children: vec![],
            errors: vec![],
        }
    }
}

impl<'input> Dom<'input> {
    pub fn parse(input: &'input str) -> Result<Self> {
        let pairs = match Grammar::parse(Rule::html, input) {
            Ok(pairs) => pairs,
            Err(error) => return formatting::error_msg(error),
        };

        Self::build_dom(pairs)
    }

    fn build_dom(pairs: Pairs<'input, Rule>) -> Result<Self> {
        let mut dom = Self::default();

        for pair in pairs {
            match pair.as_rule() {
                Rule::doctype => {
                    dom.tree_type = DomVariant::DocumentFragment;
                }
                Rule::node_element => match Self::build_node_element(pair.into_inner(), &mut dom) {
                    Ok(el) => {
                        if let Some(node) = el {
                            dom.children.push(node);
                        }
                    }
                    Err(error) => {
                        dom.errors.push(format!("{}", error));
                    }
                },
                Rule::node_text => {
                    dom.children.push(Node::Text(pair.as_str()));
                }
                Rule::EOI => break,
                _ => unreachable!("[build dom] unknown rule: {:?}", pair.as_rule()),
            };
        }

        // TODO: This needs to be cleaned up
        // What logic should apply when parsing fragment vs document?
        // I had some of this logic inside the grammar before, but i thought it would be a bit clearer
        // to just have everyting here when we construct the dom
        match dom.children.len() {
            0 => {
                dom.tree_type = DomVariant::Empty;

                Ok(dom)
            }
            1 => match dom.children[0] {
                Node::Element(ref el) => {
                    let name = el.name.to_lowercase();

                    if name == "html" {
                        dom.tree_type = DomVariant::Document;

                        Ok(dom)
                    } else if dom.tree_type == DomVariant::Document && name != "html" {
                        Err(Error::Parsing(
                            "A document can only have html as root".to_string(),
                        ))
                    } else {
                        dom.tree_type = DomVariant::DocumentFragment;

                        Ok(dom)
                    }
                }
                _ => {
                    dom.tree_type = DomVariant::DocumentFragment;

                    Ok(dom)
                }
            },
            _ => {
                dom.tree_type = DomVariant::DocumentFragment;

                for node in &dom.children {
                    if let Node::Element(ref el) = node {
                        let name = el.name.to_lowercase();

                        if name == "html" || name == "body" || name == "head" {
                            return Err(Error::Parsing(format!(
                                "A document fragment should not include {}",
                                name
                            )));
                        }
                    }
                }

                Ok(dom)
            }
        }
    }

    fn build_node_element(pairs: Pairs<'input, Rule>, dom: &mut Dom) -> Result<Option<Node<'input>>> {
        let mut element = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::node_element | Rule::el_raw_text => {
                    let element = element.get_or_insert_with(|| Element::default_with_span(pair.as_span()));

                    match Self::build_node_element(pair.into_inner(), dom) {
                        Ok(el) => {
                            if let Some(child_element) = el {
                                element.children.push(child_element)
                            }
                        }
                        Err(error) => {
                            dom.errors.push(format!("{}", error));
                        }
                    }
                }
                Rule::node_text | Rule::el_raw_text_content => {
                    let element = element.get_or_insert_with(|| Element::default_with_span(pair.as_span()));

                    element.children.push(Node::Text(pair.as_str()));
                }
                // TODO: To enable some kind of validation we should probably align this with
                // https://html.spec.whatwg.org/multipage/syntax.html#elements-2
                // Also see element variants
                Rule::el_name | Rule::el_void_name | Rule::el_raw_text_name => {
                    let element = element.get_or_insert_with(|| Element::default_with_span(pair.as_span()));

                    element.name = pair.as_str();
                }
                Rule::attr => {
                    let element = element.get_or_insert_with(|| Element::default_with_span(pair.as_span()));

                    match Self::build_attribute(pair.into_inner()) {
                        Ok((attr_key, attr_value)) => {
                            match attr_key {
                                "id" => element.id = attr_value,
                                "class" => {
                                    if let Some(classes) = attr_value {
                                        let classes = classes.split_whitespace().collect::<Vec<_>>();

                                        for class in classes {
                                            element.classes.push(class);
                                        }
                                    }
                                }
                                _ => {
                                    element.attributes.insert(attr_key, attr_value);
                                }
                            };
                        }
                        Err(error) => {
                            dom.errors.push(format!("{}", error));
                        }
                    }
                },
                Rule::el_normal_end | Rule::el_raw_text_end => {
                    let element = element.get_or_insert_with(|| Element::default_with_span(pair.as_span()));

                    element.variant = ElementVariant::Normal;

                    break;
                }
                Rule::el_dangling => (),
                Rule::EOI => (),
                _ => {
                    return Err(Error::Parsing(format!(
                        "Failed to create element at rule: {:?}",
                        pair.as_rule()
                    )))
                }
            }
        }

        let element = element.ok_or_else(|| Error::Parsing("Failed to create element".to_string()))?;

        if !element.name.is_empty() {
            Ok(Some(Node::Element(element)))
        } else {
            Ok(None)
        }
    }

    fn build_attribute(pairs: Pairs<'input, Rule>) -> Result<(&'input str, Option<&'input str>)> {
        let mut attribute = ("", None);

        for pair in pairs {
            match pair.as_rule() {
                Rule::attr_key => {
                    attribute.0 = pair.as_str();
                }
                Rule::attr_value | Rule::attr_non_quoted => {
                    attribute.1 = Some(pair.as_str());
                }
                _ => {
                    return Err(Error::Parsing(format!(
                        "Failed to create attribute at rule: {:?}",
                        pair.as_rule()
                    )))
                }
            }
        }

        Ok(attribute)
    }
}
