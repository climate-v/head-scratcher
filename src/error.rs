//! Custom error messages from NetCDF head scratcher
//!
//! - [Official nom documentation about error management](https://github.com/Geal/nom/blob/master/doc/error_management.md)
//! - [Official nom example about custom error](https://github.com/Geal/nom/blob/master/examples/custom_error.rs)
use nom::error::ErrorKind as NomErrorKind;

/// Custom error types
#[derive(Debug, PartialEq)]
pub enum HeadScratcherError<I> {
    /// Placeholder error
    EmptyError,
    /// NetCDF version is not correct
    UnsupportedNetCDFVersion,
    /// Type of list is unknown
    UnsupportedListType(u32),
    /// Expected zero, got something else
    NonZeroValue(u32),
    /// List type is 0
    UnsupportedZeroListType,
    /// Parsing UTF-8 error
    UTF8error,
    /// Unknown NetCDF data type
    UnknownNetCDFType(usize),
    /// Error caused by parsing library
    NomError(I, NomErrorKind),
    /// IO Error
    IOError(std::io::ErrorKind),
    /// Empty Variable list
    NoVariablesInFile,
    /// Empty Dimension list
    NoDimensionsInFile,
    /// Variable not in variable list
    VariableNotFound(String),
    /// Search for Dimensions unsuccessful
    CouldNotFindDimension(String),
}

impl<I> nom::error::ParseError<I> for HeadScratcherError<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        HeadScratcherError::NomError(input, kind)
    }
    fn append(_: I, _: NomErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> From<std::io::Error> for HeadScratcherError<I> {
    fn from(err: std::io::Error) -> Self {
        HeadScratcherError::IOError(err.kind())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{Err::Error, IResult};

    fn throw_custom_error(_: &[u8]) -> IResult<&[u8], &[u8], HeadScratcherError<&[u8]>> {
        Err(Error(HeadScratcherError::EmptyError))
    }

    #[test]
    fn placeholder_works() {
        let perror = throw_custom_error(b"8").unwrap_err();
        match perror {
            Error(e) => assert_eq!(e, HeadScratcherError::EmptyError),
            _ => panic!("Unexpected error: {:?}", perror),
        }
    }
}
