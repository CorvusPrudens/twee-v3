use std::fmt::Display;

#[derive(Debug)]
pub enum ParsingError<'a> {
    Parsing(&'a str),
}

impl<'a> Display for ParsingError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Parsing(error) => f.write_fmt(format_args!("{error}")),
        }
    }
}

impl<'a> std::error::Error for ParsingError<'a> {}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for ParsingError<'a> {
    fn from(value: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match value {
            nom::Err::Incomplete(_) => ParsingError::Parsing("Incomplete parsing"),
            nom::Err::Error(e) => ParsingError::Parsing(e.input),
            nom::Err::Failure(e) => ParsingError::Parsing(e.input),
        }
    }
}
