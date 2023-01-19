//! Parse twee-v3 text format, to a simple structures.
//! .twee files can be generated with Twine.
//! See [twee-3-specification.md](https://github.com/iftechfoundation/twine-specs/blob/master/twee-3-specification.md).
//!
//! ```rust
//! let twee = "your twine content";
//!
//! if let Ok(story) = twee_v3::Story::try_from(twee) {
//!     println!("{:?}", story.title());
//! }
//! ```

use std::{borrow::Cow, collections::HashMap, fmt::Display, ops::Deref};

use iter::LinkIterator;
use utils::escape_string_content;

mod error;
mod iter;
mod parser;
mod utils;

#[derive(Debug, PartialEq, Eq)]
pub struct Metadata<'a> {
    content: &'a str,
}

impl<'a> Metadata<'a> {
    pub(crate) fn new(content: &'a str) -> Self {
        Self { content }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Passage<'a> {
    title: Cow<'a, str>,
    tags: Vec<Tag<'a>>,
    metadata: Option<Metadata<'a>>,
    content: Vec<ContentNode<'a>>,
}

impl<'a> Passage<'a> {
    fn new(
        raw_title: &'a str,
        tags: Vec<Tag<'a>>,
        metadata: Option<Metadata<'a>>,
        content: Vec<ContentNode<'a>>,
    ) -> Self {
        Self {
            title: escape_string_content(raw_title),
            tags,
            metadata,
            content,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn nodes(&self) -> &[ContentNode] {
        &self.content
    }

    pub fn links(&self) -> LinkIterator {
        LinkIterator::new(&self.content)
    }
}

impl<'a> Display for Passage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.content {
            write!(f, "{node}")?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tag<'a> {
    value: Cow<'a, str>,
}

impl<'a> Display for Tag<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl<'a> Tag<'a> {
    fn new(raw_tag: &'a str) -> Self {
        Self {
            value: escape_string_content(raw_tag),
        }
    }
}

impl<'a> Deref for Tag<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref()
    }
}

impl<'a> AsRef<str> for Tag<'a> {
    fn as_ref(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ContentNode<'a> {
    Text(Cow<'a, str>),
    Link {
        text: Cow<'a, str>,
        target: Cow<'a, str>,
    },
}

impl<'a> ContentNode<'a> {
    fn text_node(text: &'a str) -> Self {
        Self::Text(escape_string_content(text))
    }

    fn link_node(text: &'a str, target: &'a str) -> Self {
        Self::Link {
            text: escape_string_content(text),
            target: escape_string_content(target),
        }
    }
}

impl<'a> Display for ContentNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentNode::Text(text) => write!(f, "{text}"),
            ContentNode::Link { text, target: _ } => write!(f, "{text}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Story<'a> {
    title: Option<Cow<'a, str>>,
    start: Option<Cow<'a, str>>,
    passages: HashMap<String, Passage<'a>>,
}

impl<'a> Story<'a> {
    fn new(
        title: Option<Cow<'a, str>>,
        start: Option<Cow<'a, str>>,
        passages: HashMap<String, Passage<'a>>,
    ) -> Self {
        Self {
            title,
            start,
            passages,
        }
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn start(&self) -> Option<&Passage> {
        match &self.start {
            Some(start) => self.passages.get(&start[..]),
            None => None,
        }
    }

    pub fn get_passage(&self, name: &str) -> Option<&Passage> {
        self.passages.get(name)
    }
}
