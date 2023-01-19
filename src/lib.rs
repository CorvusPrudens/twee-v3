use std::{borrow::Cow, collections::HashMap, fmt::Display, ops::Deref};

use parser::story::parse_story;
use utils::escape_string_content;

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
enum ContentNode<'a> {
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

    pub fn start(&self) -> Option<&str> {
        self.start.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

impl<'a> TryFrom<&'a str> for Story<'a> {
    type Error = nom::Err<nom::error::Error<&'a str>>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match parse_story(value) {
            Ok((_, story)) => Ok(story),
            Result::Err(error) => Result::Err(error),
        }
    }
}
