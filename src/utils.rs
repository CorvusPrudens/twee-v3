use std::borrow::Cow;

use nom::{
    bytes::complete::escaped_transform,
    character::complete::{anychar, char, none_of},
    error::{Error, ErrorKind, ParseError},
    Err, IResult,
};

pub(crate) fn take_delimited_greedy(
    opening_char: char,
    closing_char: char,
) -> impl Fn(&str) -> IResult<&str, &str> {
    move |i: &str| {
        // Validate that we start with the opening char.
        char(opening_char)(i)?;
        let mut index = 0;
        let mut bracket_counter = 0;

        while let Some(n) = &i[index..].find(&[opening_char, closing_char, '\\'][..]) {
            index += n;
            let mut it = i[index..].chars();
            match it.next().unwrap_or_default() {
                c if c == '\\' => {
                    // Skip the escape char `\`.
                    index += '\\'.len_utf8();
                    // Skip also the following char.
                    let c = it.next().unwrap_or_default();
                    index += c.len_utf8();
                }
                c if c == opening_char => {
                    bracket_counter += 1;
                    index += opening_char.len_utf8();
                }
                c if c == closing_char => {
                    // Closing bracket.
                    bracket_counter -= 1;
                    index += closing_char.len_utf8();
                }
                // Can not happen.
                _ => unreachable!(),
            };
            // We found the unmatched closing bracket.
            if bracket_counter == 0 {
                // We do not consume it.
                return Ok((&i[index..], &i[0..index]));
            };
        }

        if bracket_counter == 0 {
            Ok(("", i))
        } else {
            Err(Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil)))
        }
    }
}

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
