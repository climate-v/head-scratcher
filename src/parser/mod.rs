//! Main parsing module
//!
//! # Parser
//! Main parsing module.
use crate::constants_and_types as csts;
use crate::error::HeadScratcherError as HSE;
use nom::{
    bytes::streaming::tag,
    number::streaming::{be_u32, u8},
    IResult,
};

type HSEResult<I, O> = IResult<I, O, HSE<I>>;

/// NetCDF data format Type
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum NetCDFType {
    NC_BYTE,
    NC_CHAR,
    NC_SHORT,
    NC_INT,
    NC_FLOAT,
    NC_DOUBLE,
}

/// NetCDF type [atomic]
pub fn nc_type(i: &[u8]) -> HSEResult<&[u8], NetCDFType> {
    let (i, o) = be_u32(i)?;
    match o {
        csts::NC_BYTE => Ok((i, NetCDFType::NC_BYTE)),
        csts::NC_CHAR => Ok((i, NetCDFType::NC_CHAR)),
        csts::NC_SHORT => Ok((i, NetCDFType::NC_SHORT)),
        csts::NC_INT => Ok((i, NetCDFType::NC_INT)),
        csts::NC_FLOAT => Ok((i, NetCDFType::NC_FLOAT)),
        csts::NC_DOUBLE => Ok((i, NetCDFType::NC_DOUBLE)),
        _ => Err(nom::Err::Error(HSE::UnknownNetCDFType)),
    }
}

/// Number of elements [atomic]
pub fn nelems(i: &[u8]) -> HSEResult<&[u8], u32> {
    non_neg(i)
}

/// Dimension length [atomic]
pub fn non_neg(i: &[u8]) -> HSEResult<&[u8], u32> {
    be_u32(i)
}

/// Dimension length [atomic]
pub fn dim_length(i: &[u8]) -> HSEResult<&[u8], u32> {
    non_neg(i)
}

/// Get the name of an element [combined]
pub fn name(i: &[u8]) -> HSEResult<&[u8], &str> {
    let (i, count) = be_u32(i)?;
    let (i, name) = nom::bytes::streaming::take(count as usize)(i)?;
    match std::str::from_utf8(name) {
        Ok(name) => Ok((i, name)),
        Err(_) => Err(nom::Err::Error(HSE::UTF8error)),
    }
}

/// List type
#[derive(Debug, PartialEq)]
pub enum ListType {
    Absent,
    DimensionList,
    AttributeList,
    VariableList,
}

/// Decide upon the type of following list type [atomic]
pub fn list_type(i: &[u8]) -> HSEResult<&[u8], ListType> {
    let (i, o) = be_u32(i)?;
    match o {
        csts::ZERO => {
            let (i, o) = be_u32(i)?;
            if o == csts::ZERO {
                Ok((i, ListType::Absent))
            } else {
                Err(nom::Err::Error(HSE::UnsupportedZeroListType))
            }
        }
        csts::NC_DIMENSION => Ok((i, ListType::DimensionList)),
        csts::NC_VARIABLE => Ok((i, ListType::VariableList)),
        csts::NC_ATTRIBUTE => Ok((i, ListType::AttributeList)),
        _ => Err(nom::Err::Error(HSE::UnsupportedListType)),
    }
}

/// Length of record dimension
#[derive(Debug, PartialEq)]
pub enum NumberOfRecords {
    NonNegative(csts::NON_NEG),
    Streaming,
}

/// Length of record dimension [atomic]
pub fn number_of_records(i: &[u8]) -> HSEResult<&[u8], NumberOfRecords> {
    // netCDF3 uses big endian, netCDF4 needs to be checked
    let (i, o) = be_u32(i)?;
    match o {
        csts::STREAMING => Ok((i, NumberOfRecords::Streaming)),
        _ => Ok((i, NumberOfRecords::NonNegative(o))),
    }
}

/// Supported NetCDF versions
#[derive(Debug, PartialEq)]
pub enum NetCDFVersion {
    Classic,
    Offset64,
}

/// Get a single byte [atomic]
fn take_u8(i: &[u8]) -> HSEResult<&[u8], u8> {
    u8(i)
}

/// Check NetCDF initials [atomic]
pub fn initials(i: &[u8]) -> HSEResult<&[u8], &[u8]> {
    tag("CDF")(i)
}

/// Check NetCDF version [atomic]
pub fn nc_version(i: &[u8]) -> HSEResult<&[u8], NetCDFVersion> {
    let (i, o) = take_u8(i)?;
    match o {
        1 => Ok((i, NetCDFVersion::Classic)),
        2 => Ok((i, NetCDFVersion::Offset64)),
        _ => Err(nom::Err::Error(HSE::UnsupportedNetCDFVersion)),
    }
}

/// Check NetCDF magic bytes [combined]
pub fn magic(i: &[u8]) -> HSEResult<&[u8], NetCDFVersion> {
    let (i, _) = initials(i)?;
    let (i, v) = nc_version(i)?;
    Ok((i, v))
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;
    use core::panic;

    #[test]
    fn file_example_empty() {
        let i = include_bytes!("../../assets/empty.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(0));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::Absent); // No dim list
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::Absent); // No atrr list
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::Absent); // No var list
        let empty: &[u8] = &[];
        assert_eq!(i, empty)
    }

    #[test]
    fn file_example_small() {
        let i = include_bytes!("../../assets/small.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(0));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = nelems(i).unwrap();
        assert_eq!(o, 1);
        let (i, o) = name(i).unwrap();
        assert_eq!(o, "dim");
        let (i, o) = dim_length(i).unwrap();
        assert_eq!(o, 5);
    }

    #[test]
    fn file_example_1() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(1));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = nelems(i).unwrap();
        assert_eq!(o, 5);
        let (i, o) = name(i).unwrap();
        assert_eq!(o, "lat");
        let (i, o) = dim_length(i).unwrap();
        assert_eq!(o, 128)
    }

    #[test]
    fn file_example_2() {
        let i = include_bytes!("../../assets/testrh.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(0));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = nelems(i).unwrap();
        assert_eq!(o, 1);
        let (i, o) = name(i).unwrap();
        assert_eq!(o, "dim1");
    }

    #[test]
    fn file_example_3() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.3_nc64.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Offset64);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(1));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = nelems(i).unwrap();
        assert_eq!(o, 5);
        let (i, o) = name(i).unwrap();
        assert_eq!(o, "time");
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

    #[test]
    fn test_nctypes() {
        let types: [u8; 44] = [
            0, 0, 0, 3, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 2, 0,
            0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 4, 0, 0, 0, 4,
        ];
        let expected: [NetCDFType; 11] = [
            NetCDFType::NC_SHORT,
            NetCDFType::NC_CHAR,
            NetCDFType::NC_BYTE,
            NetCDFType::NC_BYTE,
            NetCDFType::NC_BYTE,
            NetCDFType::NC_SHORT,
            NetCDFType::NC_CHAR,
            NetCDFType::NC_FLOAT,
            NetCDFType::NC_DOUBLE,
            NetCDFType::NC_INT,
            NetCDFType::NC_INT,
        ];
        for (factor, exp) in expected.iter().enumerate() {
            let (_, o) = nc_type(&types[(factor * 4)..]).unwrap();
            assert_eq!(o, *exp)
        }
    }
}
