//! Parse the twee 3 interactive fiction format to simple structures.
//!
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

use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, Range},
};

use iter::LinkIterator;
use utils::escape_string_content;

mod error;
pub mod iter;
mod parser;
mod utils;

#[derive(Debug, PartialEq, Eq, Clone)]
enum TextBlock {
    Owned(String),
    Borrowed(Range<usize>),
}

impl TextBlock {
    pub fn owned(s: String) -> Self {
        Self::Owned(match escape_string_content(&s) {
            Some(escaped) => escaped,
            None => s,
        })
    }

    pub fn borrowed(original: &str, substring: &str) -> Self {
        match escape_string_content(substring) {
            // If the content is escaped, its a copy.
            Some(escaped) => Self::Owned(escaped),
            None => {
                let original_begin = original.as_ptr() as usize;
                let original_end = original_begin + original.len();
                let substring_begin = substring.as_ptr() as usize;
                let substring_end = substring_begin + substring.len();
                if substring_begin < original_begin || substring_end > original_end {
                    // substring is not a substring of original, so we need to copy it.
                    Self::Owned(substring.to_owned())
                } else {
                    Self::Borrowed(substring_begin - original_begin..substring_end - original_begin)
                }
            }
        }
    }

    pub fn as_str<'a>(&'a self, original: &'a str) -> &'a str {
        match self {
            TextBlock::Owned(s) => s.as_str(),
            TextBlock::Borrowed(r) => &original[r.clone()],
        }
    }
}

impl From<&str> for TextBlock {
    fn from(value: &str) -> Self {
        TextBlock::Owned(value.to_owned())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metadata<T> {
    content: T,
}

impl<T> Metadata<T> {
    pub(crate) fn new(content: T) -> Self {
        Self { content }
    }
}

impl Metadata<TextBlock> {
    fn as_borrowed<'a>(&'a self, original: &'a str) -> Metadata<&str> {
        Metadata::new(self.content.as_str(original))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Passage<T> {
    title: T,
    tags: Vec<Tag<T>>,
    metadata: Option<Metadata<T>>,
    content: Vec<ContentNode<T>>,
}

impl<T> Passage<T> {
    fn new(
        title: T,
        tags: Vec<Tag<T>>,
        metadata: Option<Metadata<T>>,
        content: Vec<ContentNode<T>>,
    ) -> Self {
        Self {
            title,
            tags,
            metadata,
            content,
        }
    }

    pub fn title(&self) -> &T {
        &self.title
    }

    pub fn nodes(&self) -> &[ContentNode<T>] {
        &self.content
    }

    pub fn links(&self) -> LinkIterator<T> {
        LinkIterator::new(&self.content)
    }
}

impl<T> Display for Passage<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.content {
            write!(f, "{node}")?;
        }
        Ok(())
    }
}

impl<'a> Passage<TextBlock> {
    fn as_borrowed(&'a self, original: &'a str) -> Passage<&'a str> {
        Passage {
            title: self.title.as_str(original),
            tags: self.tags.iter().map(|t| t.as_borrowed(original)).collect(),
            metadata: self.metadata.as_ref().map(|m| m.as_borrowed(original)),
            content: self
                .content
                .iter()
                .map(|n| n.as_borrowed(original))
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tag<T> {
    value: T,
}

impl<T> Display for Tag<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> Tag<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

impl Tag<TextBlock> {
    fn as_borrowed<'a>(&'a self, original: &'a str) -> Tag<&str> {
        Tag::new(self.value.as_str(original))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ContentNode<T> {
    Text(T),
    Link { text: T, target: T },
}

impl<T> ContentNode<T> {
    fn text_node(text: T) -> Self {
        Self::Text(text)
    }

    fn link_node(text: T, target: T) -> Self {
        Self::Link { text, target }
    }
}

impl<T> Display for ContentNode<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentNode::Text(text) => write!(f, "{text}"),
            ContentNode::Link { text, target: _ } => write!(f, "{text}"),
        }
    }
}

impl ContentNode<TextBlock> {
    fn as_borrowed<'a>(&'a self, original: &'a str) -> ContentNode<&str> {
        match self {
            ContentNode::Text(text) => ContentNode::Text(text.as_str(original)),
            ContentNode::Link { text, target } => ContentNode::Link {
                text: text.as_str(original),
                target: target.as_str(original),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Story<T>
where
    T: Deref<Target = str>,
{
    content: T,
    title: Option<TextBlock>,
    start: Option<TextBlock>,
    passages: HashMap<String, Passage<TextBlock>>,
}

impl<T> Story<T>
where
    T: Deref<Target = str>,
{
    fn new(
        content: T,
        title: Option<TextBlock>,
        start: Option<TextBlock>,
        passages: HashMap<String, Passage<TextBlock>>,
    ) -> Self {
        Self {
            content,
            title,
            start,
            passages,
        }
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_ref().map(|block| block.as_str(&self.content))
    }

    pub fn start(&self) -> Option<Passage<&str>> {
        self.start.as_ref().and_then(|block| {
            let start = block.as_str(&self.content);
            self.get_passage(start)
        })
    }

    pub fn get_passage(&self, name: &str) -> Option<Passage<&str>> {
        self.passages
            .get(name)
            .map(|passage| passage.as_borrowed(&self.content))
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            story: self,
            passage_names: self.passages.keys()
        }
    }
}

impl Story<&str> {
    pub fn into_owned(self) -> Story<String> {
        Story {
            content: self.content.to_owned(),
            title: self.title,
            start: self.start,
            passages: self.passages,
        }
    }
}

pub struct Iter<'a, T>
where
    T: Deref<Target = str>,
{
    story: &'a Story<T>,
    passage_names: std::collections::hash_map::Keys<'a, String, Passage<TextBlock>>
}

impl<'a, T> std::iter::Iterator for Iter<'a, T>
where
    T: Deref<Target = str>,
{
    type Item = Passage<&'a str>;
    fn next(&mut self) -> Option<Self::Item> {
        self.passage_names.next().and_then(|name| {
            self.story.get_passage(name)
        })
    }
}
