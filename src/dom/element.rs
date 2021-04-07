use super::node::Node;
use std::collections::{BTreeMap, HashMap};
use std::result::Result;
use pest::Span;

/// Normal: `<div></div>` or Void: `<meta/>`and `<meta>`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
// TODO: Align with: https://html.spec.whatwg.org/multipage/syntax.html#elements-2
pub enum ElementVariant {
    /// A normal element can have children, ex: <div></div>.
    Normal,
    /// A void element can't have children, ex: <meta /> and <meta>
    Void,
}

pub type Attributes<'input> = HashMap<&'input str, Option<&'input str>>;

/// Most of the parsed html nodes are elements, except for text
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Element<'input> {
    /// The id of the element
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub id: Option<&'input str>,

    /// The name / tag of the element
    pub name: &'input str,

    /// The element variant, if it is of type void or not
    pub variant: ElementVariant,

    /// All of the elements attributes, except id and class
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "HashMap::is_empty"))]
    #[cfg_attr(feature = "serde", serde(serialize_with = "ordered_map"))]
    pub attributes: Attributes<'input>,

    /// All of the elements classes
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub classes: Vec<&'input str>,

    /// All of the elements child nodes
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub children: Vec<Node<'input>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub span: Span<'input>,
}

impl<'input> Element<'input> {
    pub fn default_with_span(span: Span<'input>) -> Self {
        Self {
            id: None,
            name: "",
            variant: ElementVariant::Void,
            classes: vec![],
            attributes: HashMap::new(),
            children: vec![],
            span,
        }
    }
}

#[cfg(feature = "serde")]
fn ordered_map<S: serde::Serializer>(value: &Attributes, serializer: S) -> Result<S::Ok, S::Error> {
    let ordered: BTreeMap<_, _> = value.iter().collect();

    ordered.serialize(serializer)
}
