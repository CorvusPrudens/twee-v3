use std::{borrow::Cow, collections::HashMap};

use nom::{
    branch::alt,
    bytes::streaming::tag,
    character::{
        complete::{line_ending, multispace0},
        streaming::not_line_ending,
    },
    combinator::map,
    error::{Error, ErrorKind, ParseError},
    Err, IResult,
};
use serde_json::Value;

use crate::{
    parser::passage::parse_passage, utils::take_delimited_greedy, ContentNode, Metadata, Passage,
    Story, Tag, TextBlock,
};

enum StoryBlock<'a> {
    Title(&'a str),
    StoryData(StoryData2),
    Passage(Passage<&'a str>),
}

#[derive(Debug, PartialEq, Eq)]
struct StoryData<'a> {
    start: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq, Eq)]
struct StoryData2 {
    start: Option<String>,
}

fn parse_story_title(input: &str) -> IResult<&str, &str> {
    let (input, _) = nom::sequence::pair(tag(":: StoryTitle"), line_ending)(input)?;

    let (input, title) = not_line_ending(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, title))
}

fn parse_story_data(input: &str) -> IResult<&str, StoryData2> {
    let (input, _) = nom::sequence::pair(tag(":: StoryData"), line_ending)(input)?;
    let (input, data) = take_delimited_greedy('{', '}')(input)?;
    let (input, _) = multispace0(input)?;

    // Now look for start in data
    let dictionary: Value = serde_json::from_str(data)
        .map_err(|_err| Err::Error(Error::from_error_kind(input, ErrorKind::TakeUntil)))?;

    let start = dictionary
        .get("start")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    let data = StoryData2 { start };

    Ok((input, data))
}

fn parse_story_block(input: &str) -> IResult<&str, StoryBlock> {
    alt((
        map(parse_story_title, StoryBlock::Title),
        map(parse_story_data, StoryBlock::StoryData),
        map(parse_passage, StoryBlock::Passage),
    ))(input)
}

pub fn parse_story(input: &str) -> IResult<&str, Story<&str>> {
    let original = input;
    let mut title = None;
    let mut start = None;
    let mut passages = HashMap::new();

    let mut input = input;
    while !input.is_empty() {
        let (i, block) = parse_story_block(input)?;
        match block {
            StoryBlock::Title(extracted_title) => title = Some(extracted_title),
            StoryBlock::StoryData(extracted_start) => start = extracted_start.start,
            StoryBlock::Passage(passage) => {
                passages.insert(passage.title().to_string(), passage);
            }
        }
        input = i;
    }
    let title = title.map(|title| TextBlock::borrowed(original, title));
    let start = start.map(TextBlock::owned);
    let passages: HashMap<_, _> = passages
        .into_iter()
        .map(|(key, passage)| (key, passage_as_str_to_blocks(original, passage)))
        .collect();

    Ok((input, Story::new(original, title, start, passages)))
}

fn passage_as_str_to_blocks(original: &str, passage: Passage<&str>) -> Passage<TextBlock> {
    let title = TextBlock::borrowed(original, passage.title);
    let tags: Vec<_> = passage
        .tags
        .iter()
        .map(|tag| Tag::new(TextBlock::borrowed(original, tag.value)))
        .collect();
    let metadata = passage
        .metadata
        .map(|metadata| Metadata::new(TextBlock::borrowed(original, metadata.content)));
    let content: Vec<_> = passage
        .content
        .iter()
        .map(|node| match node {
            ContentNode::Text(text) => ContentNode::Text(TextBlock::borrowed(original, text)),
            ContentNode::Link { text, target } => ContentNode::Link {
                text: TextBlock::borrowed(original, text),
                target: TextBlock::borrowed(original, target),
            },
        })
        .collect();

    Passage::new(title, tags, metadata, content)
}

#[cfg(test)]
mod tests {

    use super::{parse_story, parse_story_data, parse_story_title, StoryData2};

    const TITLE_AND_DATA: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/sample/title_and_data.twee"
    ));

    const SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sample/sample.twee"));

    #[test]
    fn test_parse_story_title() {
        let input = ":: StoryTitle\nTest Story\n\n";

        assert_eq!(parse_story_title(input), Ok(("", "Test Story".into())))
    }

    #[test]
    fn test_parse_story_title_until_next_dots() {
        let input = ":: StoryTitle\nTest Story\n\n::";

        assert_eq!(parse_story_title(input), Ok(("::", "Test Story".into())))
    }

    #[test]
    fn test_parse_story_data() {
        let input =
            ":: StoryData\n{    \"ifid\": \"77599634\",\n    \"start\": \"Start story\"\n}\n\n::";

        assert_eq!(
            parse_story_data(input),
            Ok((
                "::",
                StoryData2 {
                    start: Some("Start story".into())
                }
            ))
        )
    }

    #[test]
    fn test_parse_story_just_title_and_start() {
        let input = TITLE_AND_DATA;

        let (_, story) = parse_story(input).unwrap();

        assert_eq!(Some("Test Story"), story.title());
    }

    #[test]
    fn test_parse_whole_story() {
        let input = SAMPLE;

        let (remaining_input, story) = parse_story(input).unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(Some("Test Story"), story.title());
        assert_eq!(
            Some(&"Start"),
            story.start().as_ref().map(|passage| passage.title())
        );
        assert_eq!(4, story.passages.len());

        let start = story.get_passage("Start").unwrap();
        assert_eq!(&"Start", start.title());
    }
}
