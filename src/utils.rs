use std::borrow::Cow;

use nom::{
    bytes::complete::escaped_transform,
    character::complete::{anychar, char, none_of},
    error::{Error, ErrorKind, ParseError},
    Err, FindSubstring, IResult,
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

pub(crate) fn until_link1(input: &str) -> IResult<&str, &str> {
    let mut index = 0;

    loop {
        if let Some(position) = (&input[index..]).find_substring("\\[[") {
            index += position + 2;
        } else if let Some(position) = (&input[index..]).find_substring("[[") {
            index += position;
            return if index == 0 {
                Err(Err::Error(Error::from_error_kind(
                    input,
                    ErrorKind::TakeUntil,
                )))
            } else {
                Ok((&input[index..], &input[0..index]))
            };
        } else {
            break;
        }
    }

    Ok(("", input))
}

pub(crate) fn split_escaped<'a>(input: &'a str, pat: &str) -> Option<(&'a str, &'a str)> {
    let mut index = 0;
    let escaped_pat = format!("\\{pat}");

    while let Some(position) = (&input[index..]).find_substring(&escaped_pat) {
        index += position + 2;
    }

    if let Some(position) = (&input[index..]).find_substring(pat) {
        index += position;
        Some((&input[..index], &input[index + pat.len()..]))
    } else {
        None
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

#[cfg(test)]
mod tests {
    use nom::{
        error::{Error, ErrorKind, ParseError},
        Err,
    };

    use super::{split_escaped, until_link1};

    #[test]
    fn test_until_link1() {
        let input = "This is a dog";

        assert_eq!(until_link1(input), Ok(("", "This is a dog")));
    }

    #[test]
    fn test_until_link1_has_brackets() {
        let input = "This is a dog [[bob";

        assert_eq!(until_link1(input), Ok(("[[bob", "This is a dog ")));
    }

    #[test]
    fn test_until_link1_has_escaped_brackets() {
        let input = "This is a dog \\[[bob";

        assert_eq!(until_link1(input), Ok(("", "This is a dog \\[[bob")));
    }

    #[test]
    fn test_until_link1_start_with_link() {
        let input = "[[link]]";

        assert_eq!(
            until_link1(input),
            Err(Err::Error(Error::from_error_kind(
                input,
                ErrorKind::TakeUntil,
            )))
        );
    }

    #[test]
    fn test_split_escaped() {
        let input = "hello->I'm happy";

        assert_eq!(split_escaped(input, "->"), Some(("hello", "I'm happy")));
    }

    #[test]
    fn test_split_escaped_no_match() {
        let input = "hello->I'm happy";

        assert_eq!(split_escaped(input, "|"), None);
    }

    #[test]
    fn test_split_escaped_actually_escaped() {
        let input = "hello\\->I'm happy";

        assert_eq!(split_escaped(input, "->"), None);
    }

    #[test]
    fn test_split_escaped_actually_escaped_proper_skip() {
        let input = "hello\\--I'm happy";

        assert_eq!(split_escaped(input, "-"), Some(("hello\\-", "I'm happy")));
    }
}
