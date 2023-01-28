use std::fmt::{Debug, Display};

#[derive(Debug)]
pub enum ParsingError<T> {
    Parsing(T),
}

impl<T> Display for ParsingError<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Parsing(error) => f.write_fmt(format_args!("{error}")),
        }
    }
}

impl<T> std::error::Error for ParsingError<T> where T: Display + Debug {}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for ParsingError<&'a str> {
    fn from(value: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match value {
            nom::Err::Incomplete(_) => ParsingError::Parsing("Incomplete parsing"),
            nom::Err::Error(e) => ParsingError::Parsing(e.input),
            nom::Err::Failure(e) => ParsingError::Parsing(e.input),
        }
    }
}

impl From<nom::Err<nom::error::Error<&str>>> for ParsingError<String> {
    fn from(value: nom::Err<nom::error::Error<&str>>) -> Self {
        match value {
            nom::Err::Incomplete(_) => ParsingError::Parsing("Incomplete parsing".to_string()),
            nom::Err::Error(e) => ParsingError::Parsing(e.input.to_string()),
            nom::Err::Failure(e) => ParsingError::Parsing(e.input.to_string()),
        }
    }
}
