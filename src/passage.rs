use std::{borrow::Cow, fmt::Display, ops::Deref};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::none_of,
        complete::{anychar, char, multispace0, newline, space0},
    },
    combinator::{map, opt, recognize, value},
    multi::{many1_count, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::{
    metadata::{parse_metadata, Metadata},
    utils::{escape_string_content, split_escaped, take_until_pattern, until_link1},
};

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

fn parse_escaped_char(input: &str) -> IResult<&str, char> {
    preceded(char('\\'), anychar)(input)
}

fn parse_tag(input: &str) -> IResult<&str, Tag> {
    let parse_tag = recognize(many1_count(alt((parse_escaped_char, none_of(" ]")))));
    map(parse_tag, Tag::new)(input)
}

pub fn parse_tags(input: &str) -> IResult<&str, Vec<Tag>> {
    let each_tags = separated_list0(tag(" "), parse_tag);

    let mut parse_tags = delimited(tag("["), each_tags, tag("]"));
    parse_tags(input)
}

fn parse_title(input: &str) -> IResult<&str, &str> {
    let parse_word = recognize(many1_count(alt((parse_escaped_char, none_of(" \n\r[{")))));

    let title_block = recognize(separated_list1(tag(" "), value((), parse_word)));

    preceded(tag(":: "), title_block)(input)
}

fn find_content_block(input: &str) -> IResult<&str, &str> {
    take_until_pattern(preceded(newline, tag("::")))(input)
}

fn parse_text_node(input: &str) -> IResult<&str, ContentNode> {
    let (input, text) = until_link1(input)?;
    Ok((input, ContentNode::text_node(text)))
}

fn parse_link_node<'a>(input: &'a str) -> IResult<&str, ContentNode> {
    let parse_link_content = recognize(many1_count(alt((parse_escaped_char, none_of("\n\r]")))));

    let (input, link_content) = delimited(tag("[["), parse_link_content, tag("]]"))(input)?;

    let piped = |link_content| split_escaped(link_content, "|");
    let to_right = |link_content| split_escaped(link_content, "->");
    let to_left =
        |link_content| split_escaped(link_content, "<-").map(|(target, text)| (text, target));
    let simple = |link_content: &'a str| -> (&str, &str) { (link_content, link_content) };

    let (text, target) = piped(link_content)
        .or_else(|| to_right(link_content))
        .or_else(|| to_left(link_content))
        .unwrap_or_else(|| simple(link_content));

    Ok((input, ContentNode::link_node(text, target)))
}

fn parse_node(input: &str) -> IResult<&str, ContentNode> {
    alt((parse_text_node, parse_link_node))(input)
}

pub fn parse_passage(input: &str) -> IResult<&str, Passage> {
    let (input, title) = parse_title(input)?;
    let (input, _) = space0(input)?;
    let (input, tags) = opt(parse_tags)(input)?;
    let (input, _) = space0(input)?;
    let (input, metadata) = opt(parse_metadata)(input)?;
    let (input, _) = recognize(pair(space0, newline))(input)?;
    let (input, content) = find_content_block(input)?;
    let (input, _) = multispace0(input)?;

    let mut nodes = vec![];
    let mut content = content.trim_end_matches(&['\r', '\n']);
    while !content.is_empty() {
        let (c, node) = parse_node(content)?;
        nodes.push(node);
        content = c;
    }

    Ok((
        input,
        Passage::new(title, tags.unwrap_or_default(), metadata, nodes),
    ))
}

#[cfg(test)]
mod tests {
    use nom::{
        error::{Error, ErrorKind, ParseError},
        Err,
    };

    use crate::{
        metadata::Metadata,
        passage::{find_content_block, parse_passage, parse_tags, parse_title, Passage, Tag},
    };

    use super::{parse_link_node, parse_text_node, ContentNode};

    #[test]
    fn test_tags() {
        let input = "[hello tag]";

        assert_eq!(
            parse_tags(input),
            Ok(("", vec![Tag::new("hello"), Tag::new("tag")]))
        );
    }

    #[test]
    fn test_tags_escaped() {
        let input = r"[hello\] tag]";

        assert_eq!(
            parse_tags(input),
            Ok(("", vec![Tag::new(r"hello\]"), Tag::new("tag")]))
        );
    }

    #[test]
    fn test_tags_escaped_and_dash() {
        let input = r"[hello\[-\]tag how-are you]";

        assert_eq!(
            parse_tags(input),
            Ok((
                "",
                vec![
                    Tag::new(r"hello\[-\]tag"),
                    Tag::new("how-are"),
                    Tag::new("you")
                ]
            ))
        );
    }

    #[test]
    fn test_tag_from_escapable_string_get_escaped() {
        let tag = Tag::new(r"test\]");

        assert_eq!("test]", tag.as_ref());
    }

    #[test]
    fn test_title() {
        let story = ":: Hello, this is a title";

        assert_eq!(parse_title(story), Ok(("", "Hello, this is a title")));
    }

    #[test]
    fn test_title_with_tags() {
        let story = ":: Hello, this is a title [tag1 tag2]";

        assert_eq!(
            parse_title(story),
            Ok((" [tag1 tag2]", "Hello, this is a title"))
        );
    }

    #[test]
    fn test_title_with_metadata() {
        let input = r#":: Hello, this is a title {"position":"600,400","size":"100,200"}"#;

        assert_eq!(
            parse_title(input),
            Ok((
                r#" {"position":"600,400","size":"100,200"}"#,
                "Hello, this is a title"
            ))
        );
    }

    #[test]
    fn test_title_start_with_whitespace() {
        let input = r":: \ Second [tag]";
        assert_eq!(parse_title(input), Ok((r#" [tag]"#, r"\ Second")));
    }

    #[test]
    fn test_passage() {
        let input = ":: Hello, this is a title [tag1 tag2]\n";

        let expected = Passage::new(
            "Hello, this is a title",
            vec![Tag::new("tag1"), Tag::new("tag2")],
            None,
            vec![],
        );

        assert_eq!(parse_passage(input), Ok(("", expected)));
    }

    #[test]
    fn test_passage_tag_and_metadata() {
        let input =
            ":: Hello, this is a title [tag1 tag2] {\"position\":\"900,600\",\"size\":\"200,200\"}\n";

        let expected = Passage::new(
            "Hello, this is a title",
            vec![Tag::new("tag1"), Tag::new("tag2")],
            Some(Metadata::new(r#"{"position":"900,600","size":"200,200"}"#)),
            vec![],
        );

        assert_eq!(parse_passage(input), Ok(("", expected)));
    }

    #[test]
    fn test_passage_til_next() {
        let input = ":: Some title\nHello\n\n:: Other title";

        let result = parse_passage(input);

        println!("{result:?}");
    }

    #[test]
    fn test_find_content_block() {
        let input = "Hello\n\n:: Other title";

        let result = find_content_block(input);

        assert_eq!(result, Ok(("\n:: Other title", "Hello\n")));
    }

    #[test]
    fn test_parse_text_node() {
        let input = "Hello\nThis is text[[link]]";

        assert_eq!(
            parse_text_node(input),
            Ok(("[[link]]", ContentNode::text_node("Hello\nThis is text")))
        );
    }

    #[test]
    fn test_parse_text_node_escaped_bracket() {
        let input = "Hello\nThis is text\\[[link]]";

        assert_eq!(
            parse_text_node(input),
            Ok(("", ContentNode::text_node(input)))
        );
    }

    #[test]
    fn test_parse_text_node_is_link_node() {
        let input = "[[link]]";

        assert_eq!(
            parse_text_node(input),
            Err(Err::Error(Error::from_error_kind(
                input,
                ErrorKind::TakeUntil,
            )))
        );
    }

    #[test]
    fn test_parse_link_node_simple() {
        let input = "[[link]]";

        assert_eq!(
            parse_link_node(input),
            Ok(("", ContentNode::link_node("link", "link")))
        )
    }

    #[test]
    fn test_parse_link_node_pipe() {
        let input = "[[first|First]]";

        assert_eq!(
            parse_link_node(input),
            Ok(("", ContentNode::link_node("first", "First")))
        )
    }

    #[test]
    fn test_parse_link_node_to_right() {
        let input = "[[some text->First page]]";

        assert_eq!(
            parse_link_node(input),
            Ok(("", ContentNode::link_node("some text", "First page")))
        )
    }

    #[test]
    fn test_parse_link_node_to_left() {
        let input = "[[A page<-going somewhere?]]";

        assert_eq!(
            parse_link_node(input),
            Ok(("", ContentNode::link_node("going somewhere?", "A page")))
        )
    }
}
