use crate::ContentNode;

pub struct LinkIterator<'a> {
    nodes: &'a [ContentNode<'a>],
}

impl<'a> LinkIterator<'a> {
    pub fn new(nodes: &'a [ContentNode<'a>]) -> Self {
        Self { nodes }
    }
}

impl<'a> Iterator for LinkIterator<'a> {
    type Item = Link<'a>;

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

pub struct Link<'a> {
    pub text: &'a str,
    pub target: &'a str,
}
