//! Custom error messages from NetCDF head scratcher
//!
//! - [Official nom documentation about error management](https://github.com/Geal/nom/blob/master/doc/error_management.md)
//! - [Official nom example about custom error](https://github.com/Geal/nom/blob/master/examples/custom_error.rs)
use nom::error::ErrorKind as NomErrorKind;

/// Custom error types
#[derive(Debug, PartialEq)]
pub enum HeadScratcherError<I> {
    EmptyError,
    UnsupportedNetCDFVersion,
    UnsupportedListType,
    UnsupportedZeroListType,
    NomError(I, NomErrorKind),
}

impl<I> nom::error::ParseError<I> for HeadScratcherError<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        HeadScratcherError::NomError(input, kind)
    }
    fn append(_: I, _: NomErrorKind, other: Self) -> Self {
        other
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
