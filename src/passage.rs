use std::{borrow::Cow, fmt::Display, ops::Deref};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::{
        complete::none_of,
        complete::{anychar, char, newline, space0},
    },
    combinator::{map, opt, recognize, value},
    multi::{many1_count, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded},
    IResult,
};

use crate::{
    metadata::{parse_metadata, Metadata},
    utils::escape_string_content,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Passage<'a> {
    title: Cow<'a, str>,
    tags: Vec<Tag<'a>>,
    metadata: Option<Metadata<'a>>,
}

impl<'a> Passage<'a> {
    fn new(raw_title: &'a str, tags: Vec<Tag<'a>>, metadata: Option<Metadata<'a>>) -> Self {
        Self {
            title: escape_string_content(raw_title.trim_end()),
            tags,
            metadata,
        }
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

pub fn parse_passage(input: &str) -> IResult<&str, Passage> {
    let (input, title) = parse_title(input)?;
    let (input, _) = space0(input)?;
    let (input, tags) = opt(parse_tags)(input)?;
    let (input, _) = space0(input)?;
    let (input, metadata) = opt(parse_metadata)(input)?;
    let (input, _) = recognize(pair(space0, newline))(input)?;

    Ok((
        input,
        Passage::new(title, tags.unwrap_or_default(), metadata),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        metadata::Metadata,
        passage::{parse_passage, parse_tags, parse_title, Passage, Tag},
    };

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
        );

        assert_eq!(parse_passage(input), Ok(("", expected)));
    }
}
