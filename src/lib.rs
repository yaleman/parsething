use std::{io::BufRead, string::FromUtf8Error};

pub mod gguf;
pub mod prelude;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    InvalidFormat,
    UnsupportedVersion,
    CorruptedData,
    InvalidData,
    InvalidHeader(String),
    InvalidUtf8(FromUtf8Error),
    /// We tried to read more bytes than were available.
    NeedMoreBytes,
}

impl From<FromUtf8Error> for ParseError {
    fn from(err: FromUtf8Error) -> Self {
        ParseError::InvalidUtf8(err)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(_err: std::io::Error) -> Self {
        ParseError::NeedMoreBytes
    }
}

pub trait ParseThing {
    fn parse(data: &mut impl BufRead) -> Result<Box<Self>, ParseError>;

    fn verify(&self) -> Result<bool, ParseError>;
}
