use std::borrow::Cow;

use nom::{
    bytes::complete::escaped_transform,
    character::complete::{anychar, none_of},
    IResult,
};

pub(crate) fn escape_string_content(input: &str) -> Cow<str> {
    fn escape_replace(input: &str) -> IResult<&str, String> {
        escaped_transform(none_of(r"\"), '\\', anychar)(input)
    }

    if input.contains('\\') {
        escape_replace(input).map_or_else(
            |_| Cow::Borrowed(input),
            |(_, replaced)| Cow::Owned(replaced),
        )
    } else {
        Cow::Borrowed(input)
    }
}
