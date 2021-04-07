use super::element::Element;

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'input> {
    Text(&'input str),
    Element(Element<'input>),
    Comment(&'input str),
}

impl<'a, 'input> IntoIterator for &'a Node<'input> {
    type Item = &'a Node<'input>;
    type IntoIter = NodeIntoIterator<'a, 'input>;

    fn into_iter(self) -> Self::IntoIter {
        NodeIntoIterator {
            node: self,
            index: vec![],
        }
    }
}

pub struct NodeIntoIterator<'a, 'input> {
    node: &'a Node<'input>,
    // We add/remove to this vec each time we go up/down a node three
    index: Vec<(usize, &'a Node<'input>)>,
}

impl<'a, 'input> Iterator for NodeIntoIterator<'a, 'input> {
    type Item = &'a Node<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get first child
        let child = match self.node {
            Node::Element(ref e) => e.children.get(0),
            _ => None,
        };

        let result = match child {
            // If element has child, return child
            Some(child) => {
                self.index.push((0, self.node));
                self.node = child;

                Some(child)
            }
            // If element doesn't have a child, but is a child of another node
            None if !self.index.is_empty() => {
                let mut has_finished = false;
                let mut next_node = None;

                while !has_finished {
                    // Try to get the next sibling of the parent node
                    if let Some((sibling_index, parent)) = self.index.pop() {
                        let next_sibling = sibling_index + 1;

                        let sibling = if let Node::Element(ref e) = parent {
                            e.children.get(next_sibling)
                        } else {
                            None
                        };

                        if sibling.is_some() {
                            has_finished = true;

                            self.index.push((next_sibling, parent));

                            next_node = sibling;
                        } else {
                            continue;
                        }
                    // Break of there are no more parents
                    } else {
                        has_finished = true;
                    }
                }

                if let Some(next_node) = next_node {
                    self.node = next_node;
                }

                next_node
            }
            _ => None,
        };

        result
    }
}
