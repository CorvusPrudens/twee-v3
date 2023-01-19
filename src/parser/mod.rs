use crate::{error::ParsingError, Story};

use self::story::parse_story;

pub(crate) mod metadata;
pub(crate) mod passage;
pub(crate) mod story;

impl<'a> TryFrom<&'a str> for Story<'a> {
    type Error = ParsingError<'a>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match parse_story(value) {
            Ok((_, story)) => Ok(story),
            Result::Err(error) => Result::Err(error.into()),
        }
    }
}
