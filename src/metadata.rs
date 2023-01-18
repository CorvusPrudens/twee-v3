use nom::IResult;

use crate::utils::take_delimited_greedy;

#[derive(Debug, PartialEq, Eq)]
pub struct Metadata<'a> {
    content: &'a str,
}

impl<'a> Metadata<'a> {
    pub(crate) fn new(content: &'a str) -> Self {
        Self { content }
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
