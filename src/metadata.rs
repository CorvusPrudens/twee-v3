use nom::{
    character::complete::char,
    error::{Error, ErrorKind, ParseError},
    Err, IResult,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Metadata<'a> {
    content: &'a str,
}

impl<'a> Metadata<'a> {
    pub(crate) fn new(content: &'a str) -> Self {
        Self { content }
    }
}

pub fn take_delimited_greedy(
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

pub fn parse_metadata(input: &str) -> IResult<&str, Metadata> {
    let (input, content) = take_delimited_greedy('{', '}')(input)?;
    Ok((input, Metadata::new(content)))
}

#[cfg(test)]
mod tests {
    use super::{parse_metadata, take_delimited_greedy, Metadata};

    #[test]
    fn test_take_greedy_simple_metadata() {
        let input = r#"{"position":"900,600","size":"200,200"}"#;

        assert_eq!(take_delimited_greedy('{', '}')(input), Ok(("", input)));
    }

    #[test]
    fn test_take_greedy_escaped_brackets() {
        let input = r#"{"name":"I'm \{ joe","birth":"20 of July"}"#;

        assert_eq!(take_delimited_greedy('{', '}')(input), Ok(("", input)));
    }

    #[test]
    fn test_metadata_reminder() {
        let input = r#"{"position":"900,600","size":"200,200"} and some other stuff"#;

        let expected_metadata = Metadata {
            content: r#"{"position":"900,600","size":"200,200"}"#,
        };

        assert_eq!(
            parse_metadata(input),
            Ok((" and some other stuff", expected_metadata))
        );
    }
}
