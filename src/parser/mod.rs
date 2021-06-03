//! Main parsing module
//!
//! # Parser
//! Main parsing module.
use crate::error::HeadScratcherError as HSE;
use nom::{IResult, bytes::streaming::tag, number::streaming::{u32, u8}};
use constants_and_types as csts;
pub mod constants_and_types;

/// Length of record dimension
#[derive(Debug, PartialEq)]
pub enum NumberOfRecords {
    NonNegative(csts::NON_NEG),
    Streaming,
}

/// Length of record dimension
pub fn number_of_records(i: &[u8]) -> IResult<&[u8], NumberOfRecords, HSE<&[u8]>> {
    // netCDF3 uses big endian, netCDF4 needs to be checked
    let (i, o) = u32(nom::number::Endianness::Big)(i)?;
    match o {
        csts::STREAMING => Ok((i, NumberOfRecords::Streaming)),
        _ => Ok((i, NumberOfRecords::NonNegative(o)))
    }
}

/// Supported NetCDF versions
#[derive(Debug, PartialEq)]
pub enum NetCDFVersion {
    Classic,
    Offset64,
}

/// Get a single byte
fn take_u8(i: &[u8]) -> IResult<&[u8], u8, HSE<&[u8]>> {
    u8(i)
}

/// Check NetCDF initials [atomic]
pub fn initials(i: &[u8]) -> IResult<&[u8], &[u8], HSE<&[u8]>> {
    tag("CDF")(i)
}

/// Check NetCDF version [atomic]
pub fn nc_version(i: &[u8]) -> IResult<&[u8], NetCDFVersion, HSE<&[u8]>> {
    let (i, o) = take_u8(i)?;
    match o {
        1 => Ok((i, NetCDFVersion::Classic)),
        2 => Ok((i, NetCDFVersion::Offset64)),
        _ => Err(nom::Err::Error(HSE::UnsupportedNetCDFVersion)),
    }
}

/// Check NetCDF magic bytes [combined]
pub fn magic(i: &[u8]) -> IResult<&[u8], NetCDFVersion, HSE<&[u8]>> {
    let (i, _) = initials(i)?;
    let (i, v) = nc_version(i)?;
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use core::panic;
    use super::*;

    #[test]
    fn file_example_1() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (_i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(1))
    }

    #[test]
    fn file_example_2() {
        let i = include_bytes!("../../assets/testrh.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (_i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(0))
    }

    #[test]
    fn file_example_3() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.3_nc64.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Offset64);
        let (_i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(1))
    }

    #[test]
    fn test_size() {
        let data = [0x0, 0x0, 0x0, 0xAu8];
        let (_, o) = number_of_records(&data).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(10));
        let data = [0xFF, 0xFF, 0xFF, 0xFFu8];
        let (_, o) = number_of_records(&data).unwrap();
        assert_eq!(o, NumberOfRecords::Streaming)
    }

    #[test]
    #[should_panic]
    fn test_initials_hdf() {
        let f = include_bytes!("../../assets/test_hgroups.nc");
        magic(f).unwrap();
    }

    #[test]
    fn test_wrong_version() {
        let f = include_bytes!("../../assets/test_hgroups.nc");
        let e = nc_version(f).unwrap_err();
        match e {
            nom::Err::Error(e) => assert_eq!(e, HSE::UnsupportedNetCDFVersion),
            _ => panic!("Unexpected error {:?}", e),
        }
    }

    #[test]
    fn test_nc_version() {
        let (_, o) = nc_version(&[1u8]).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (_, o) = nc_version(&[2u8]).unwrap();
        assert_eq!(o, NetCDFVersion::Offset64);
    }
}
