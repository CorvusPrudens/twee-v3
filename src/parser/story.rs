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
    parser::passage::parse_passage,
    utils::{escape_string_content, take_delimited_greedy},
    Passage, Story,
};

enum StoryBlock<'a> {
    Title(Cow<'a, str>),
    StoryData(StoryData<'a>),
    Passage(Passage<'a>),
}

#[derive(Debug, PartialEq, Eq)]
struct StoryData<'a> {
    start: Option<Cow<'a, str>>,
}

fn parse_story_title(input: &str) -> IResult<&str, Cow<str>> {
    let (input, _) = nom::sequence::pair(tag(":: StoryTitle"), line_ending)(input)?;

    let (input, title) = not_line_ending(input)?;
    let (input, _) = multispace0(input)?;

    Ok((input, escape_string_content(title)))
}

fn parse_story_data(input: &str) -> IResult<&str, StoryData> {
    let (input, _) = nom::sequence::pair(tag(":: StoryData"), line_ending)(input)?;
    let (input, data) = take_delimited_greedy('{', '}')(input)?;
    let (input, _) = multispace0(input)?;

    // Now look for start in data
    let dictionary: Value = serde_json::from_str(data)
        .map_err(|_err| Err::Error(Error::from_error_kind(input, ErrorKind::TakeUntil)))?;

    let start = dictionary
        .get("start")
        .and_then(|value| value.as_str())
        .map(|value| Cow::Owned(escape_string_content(value).into_owned()));

    let data = StoryData { start };

    Ok((input, data))
}

fn parse_story_block(input: &str) -> IResult<&str, StoryBlock> {
    alt((
        map(parse_story_title, StoryBlock::Title),
        map(parse_story_data, StoryBlock::StoryData),
        map(parse_passage, StoryBlock::Passage),
    ))(input)
}

pub fn parse_story(input: &str) -> IResult<&str, Story> {
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

    Ok((input, Story::new(title, start, passages)))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{parse_story, parse_story_data, parse_story_title, Story, StoryData};

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
                StoryData {
                    start: Some("Start story".into())
                }
            ))
        )
    }

    #[test]
    fn test_parse_story_just_title_and_start() {
        let input = TITLE_AND_DATA;

        assert_eq!(
            parse_story(input),
            Ok((
                "",
                Story::new(
                    Some("Test Story".into()),
                    Some("Start".into()),
                    HashMap::new()
                )
            ))
        )
    }

    #[test]
    fn test_parse_whole_story() {
        let input = SAMPLE;

        let (remaining_input, story) = parse_story(input).unwrap();
        assert_eq!(remaining_input, "");
        assert_eq!(Some("Test Story"), story.title());
        assert_eq!(Some("Start"), story.start());
        assert_eq!(4, story.passages.len());

        let start = &story.passages["Start"];
        assert_eq!("Start", start.title());
    }
}
