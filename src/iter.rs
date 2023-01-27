use crate::ContentNode;

pub struct LinkIterator<'a, T> {
    nodes: &'a [ContentNode<T>],
}

impl<'a, T> LinkIterator<'a, T> {
    pub fn new(nodes: &'a [ContentNode<T>]) -> Self {
        Self { nodes }
    }
}

impl<'a, T> Iterator for LinkIterator<'a, T> {
    type Item = Link<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let link;
        loop {
            if self.nodes.is_empty() {
                return None;
            }
            let node = &self.nodes[0];
            if let ContentNode::Link { text, target } = node {
                link = Some(Link { text, target });
                break;
            }
            self.nodes = &self.nodes[1..];
        }
        self.nodes = &self.nodes[1..];
        link
    }
}

pub struct Link<'a, T> {
    pub text: &'a T,
    pub target: &'a T,
}
