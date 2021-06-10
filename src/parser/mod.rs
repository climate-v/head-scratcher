//! Main parsing module
//!
//! # Parser
//! Main parsing module.
use crate::constants_and_types as csts;
use crate::error::HeadScratcherError as HSE;
use nom::{
    bytes::streaming::tag,
    number::streaming::{be_u32, be_u64, u8},
    IResult,
};

type HSEResult<I, O> = IResult<I, O, HSE<I>>;

/// NetCDF Variable
#[derive(Debug, PartialEq)]
pub struct NetCDFVariable {
    name: String,
    dims: Vec<u32>,
    attributes: Option<Vec<NetCDFAttribute>>,
    nc_type: NetCDFType,
    vsize: usize,
    begin: u64,
}

impl NetCDFVariable {
    /// Generate a new variable
    pub fn new(
        name: String,
        dims: Vec<u32>,
        attributes: Option<Vec<NetCDFAttribute>>,
        nc_type: NetCDFType,
        vsize: usize,
        begin: u64,
    ) -> Self {
        NetCDFVariable {
            name,
            dims,
            attributes,
            nc_type,
            vsize,
            begin,
        }
    }
}

/// Parse a single NetCDF variable [combined]
pub fn variable(i: &[u8], version: NetCDFVersion) -> HSEResult<&[u8], NetCDFVariable> {
    let (i, name) = name(i)?;
    let (i, dims) = nom::multi::length_count(nelems, nelems)(i)?;
    let (mut i, attr_present) = list_type(i)?;
    let attrs = match attr_present {
        ListType::Absent => None,
        _ => {
            let (k, attrs) = attribute_list(i)?;
            i = k;
            Some(attrs)
        }
    };
    let (i, nc_type) = nc_type(i)?;
    let (mut i, vsize) = nelems(i)?;
    let begin = match version {
        NetCDFVersion::Classic => {
            let (k, r) = be_u32(i)?;
            i = k;
            r as u64
        }
        NetCDFVersion::Offset64 => {
            let (k, r) = be_u64(i)?;
            i = k;
            r as u64
        }
    };
    let var = NetCDFVariable::new(
        name.to_string(),
        dims,
        attrs,
        nc_type,
        vsize as usize,
        begin,
    );
    Ok((i, var))
}

/// Parse a list of NetCDF variables [combined]
pub fn variable_list(i: &[u8], version: NetCDFVersion) -> HSEResult<&[u8], Vec<NetCDFVariable>> {
    let (mut i, mut count) = nelems(i)?;
    let mut result: Vec<NetCDFVariable> = Vec::new();
    while count > 0 {
        let (k, v) = variable(i, version)?;
        result.push(v);
        count -= 1;
        i = k;
    }
    Ok((i, result))
}

/// NetCDF Attribute
#[derive(Debug, PartialEq)]
pub struct NetCDFAttribute {
    name: String,
    nc_type: NetCDFType,
    data: Vec<u8>, // TODO Add support for proper data
}

impl NetCDFAttribute {
    /// Create a new NetCDF Attribute
    pub fn new(name: String, nc_type: NetCDFType, data: Vec<u8>) -> Self {
        NetCDFAttribute {
            name,
            nc_type,
            data,
        }
    }
}

/// Parse a single NetCDF attribute [combined]
pub fn attribute(i: &[u8]) -> HSEResult<&[u8], NetCDFAttribute> {
    let (i, (name, nc_type, nelems)) = nom::sequence::tuple((name, nc_type, nelems))(i)?;
    let (i, data) = nom::bytes::streaming::take(nelems)(i)?;
    // names are padded to the next 4-byte boundary
    let drop = padding(nelems);
    let (i, _) = nom::bytes::streaming::take(drop as u8)(i)?;
    let result = NetCDFAttribute::new(name.to_string(), nc_type, data.to_vec());
    Ok((i, result))
}

/// Parse a list of NetCDF attributes [combined]
pub fn attribute_list(i: &[u8]) -> HSEResult<&[u8], Vec<NetCDFAttribute>> {
    nom::multi::length_count(nelems, attribute)(i)
    // TODO return HashMap instead of VectorList
}

/// NetCDF Dimension
#[derive(Debug, PartialEq)]
pub struct NetCDFDimension {
    name: String,
    length: usize,
}

impl NetCDFDimension {
    /// Create new NetCDF dimension
    pub fn new(name: String, length: usize) -> Self {
        NetCDFDimension { name, length }
    }
}

/// Parse a single NetCDF dimension [combined]
pub fn dimension(i: &[u8]) -> HSEResult<&[u8], NetCDFDimension> {
    let (i, (name, dim_length)) = nom::sequence::tuple((name, dim_length))(i)?;
    let ncdim = NetCDFDimension::new(name.to_string(), dim_length as usize);
    Ok((i, ncdim))
}

/// Parse a list of NetCDF dimensions [combined]
pub fn dimension_list(i: &[u8]) -> HSEResult<&[u8], Vec<NetCDFDimension>> {
    nom::multi::length_count(nelems, dimension)(i)
    // TODO return HashMap instead of VectorList
}

/// NetCDF data format types
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

impl NetCDFType {
    /// Get the external size of type in bytes
    pub fn extsize(&self) -> usize {
        match self {
            NetCDFType::NC_BYTE => 1,
            NetCDFType::NC_CHAR => 1,
            NetCDFType::NC_SHORT => 2,
            NetCDFType::NC_INT => 4,
            NetCDFType::NC_FLOAT => 4,
            NetCDFType::NC_DOUBLE => 8,
        }
    }
}

/// Parse NetCDF data format types [atomic]
pub fn nc_type(i: &[u8]) -> HSEResult<&[u8], NetCDFType> {
    let (i, o) = be_u32(i)?;
    match o {
        csts::NC_BYTE => Ok((i, NetCDFType::NC_BYTE)),
        csts::NC_CHAR => Ok((i, NetCDFType::NC_CHAR)),
        csts::NC_SHORT => Ok((i, NetCDFType::NC_SHORT)),
        csts::NC_INT => Ok((i, NetCDFType::NC_INT)),
        csts::NC_FLOAT => Ok((i, NetCDFType::NC_FLOAT)),
        csts::NC_DOUBLE => Ok((i, NetCDFType::NC_DOUBLE)),
        _ => Err(nom::Err::Error(HSE::UnknownNetCDFType(o as usize))),
    }
}

/// Parse number of elements [atomic]
pub fn nelems(i: &[u8]) -> HSEResult<&[u8], u32> {
    non_neg(i)
}

/// Parse non negative numbers [atomic]
pub fn non_neg(i: &[u8]) -> HSEResult<&[u8], u32> {
    be_u32(i)
}

/// Parse dimension length [atomic]
pub fn dim_length(i: &[u8]) -> HSEResult<&[u8], u32> {
    non_neg(i)
}

/// Calculate padding to the next 4-byte boundary
fn padding(count: u32) -> u8 {
    let pad = 4 - (count % 4);
    match pad {
        4 => 0,
        _ => pad as u8,
    }
}

/// Parse the name of an element (dimension, variable, or attribute) [combined]
pub fn name(i: &[u8]) -> HSEResult<&[u8], &str> {
    let (i, count) = be_u32(i)?;
    let (i, name) = nom::bytes::streaming::take(count as usize)(i)?;

    // names are padded to the next 4-byte boundary
    let drop = padding(count);
    let (i, _) = nom::bytes::streaming::take(drop as u8)(i)?;

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

/// Parse a zero [atomic]
pub fn zero(i: &[u8]) -> HSEResult<&[u8], bool> {
    let (i, o) = be_u32(i)?;
    match o {
        0 => Ok((i, true)),
        _ => Err(nom::Err::Error(HSE::NonZeroValue(o))),
    }
}

/// Parse an absent list [combined]
pub fn absent(i: &[u8]) -> HSEResult<&[u8], ListType> {
    let (i, _) = nom::sequence::tuple((zero, zero))(i)?;
    Ok((i, ListType::Absent))
}

/// Parse upcoming list type [atomic]
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
        _ => Err(nom::Err::Error(HSE::UnsupportedListType(o))),
    }
}

/// Length of record dimension
#[derive(Debug, PartialEq)]
pub enum NumberOfRecords {
    NonNegative(csts::NON_NEG),
    Streaming,
}

/// Parse length of record dimension [atomic]
pub fn number_of_records(i: &[u8]) -> HSEResult<&[u8], NumberOfRecords> {
    // netCDF3 uses big endian, netCDF4 needs to be checked
    let (i, o) = be_u32(i)?;
    match o {
        csts::STREAMING => Ok((i, NumberOfRecords::Streaming)),
        _ => Ok((i, NumberOfRecords::NonNegative(o))),
    }
}

/// Supported NetCDF versions
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NetCDFVersion {
    Classic,
    Offset64,
}

/// Parse a single byte [atomic]
fn take_u8(i: &[u8]) -> HSEResult<&[u8], u8> {
    u8(i)
}

/// Parse NetCDF initials [atomic]
pub fn initials(i: &[u8]) -> HSEResult<&[u8], &[u8]> {
    tag("CDF")(i)
}

/// Parse NetCDF version [atomic]
pub fn nc_version(i: &[u8]) -> HSEResult<&[u8], NetCDFVersion> {
    let (i, o) = take_u8(i)?;
    match o {
        1 => Ok((i, NetCDFVersion::Classic)),
        2 => Ok((i, NetCDFVersion::Offset64)),
        _ => Err(nom::Err::Error(HSE::UnsupportedNetCDFVersion)),
    }
}

/// Parse NetCDF magic bytes [combined]
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
        let (i, o) = dimension_list(i).unwrap();
        let d = vec![NetCDFDimension::new("dim".to_string(), 5)];
        assert_eq!(o, d);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::Absent);
    }

    #[test]
    fn file_example_compressible() {
        let i = include_bytes!("../../assets/testrh.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(0));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = dimension_list(i).unwrap();
        let d = vec![NetCDFDimension::new("dim1".to_string(), 10_000)];
        assert_eq!(o, d);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::Absent);
    }

    #[test]
    fn file_nc3_classic() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, v) = nc_version(i).unwrap();
        assert_eq!(v, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(1));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = dimension_list(i).unwrap();
        let d = vec![
            NetCDFDimension::new("lat".to_string(), 128),
            NetCDFDimension::new("lon".to_string(), 256),
            NetCDFDimension::new("bnds".to_string(), 2),
            NetCDFDimension::new("plev".to_string(), 17),
            NetCDFDimension::new("time".to_string(), 0), // TODO Should this be the length in NoR?
        ];
        assert_eq!(o, d);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::AttributeList);
        let (i, o) = attribute_list(i).unwrap();
        assert_eq!(o.len(), 18);
        let a = NetCDFAttribute::new(
            "CVS_Id".to_string(),
            NetCDFType::NC_CHAR,
            vec![36, 73, 100, 36],
        );
        assert_eq!(o[0], a);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::VariableList);
        let (i, o) = variable_list(i, v).unwrap();
        assert_eq!(o[0].name, "area");
        assert_eq!(o[0].dims, vec![0, 1]);
        assert_eq!(o[0].begin, 7564);
        assert_eq!(o[0].vsize, 131072);
        assert_eq!(o[0].nc_type, NetCDFType::NC_FLOAT);
        // TODO check other variables
    }

    #[test]
    fn file_nc3_64offset() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.3_nc64.nc");
        let (i, o) = initials(i).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Offset64);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(1));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = dimension_list(i).unwrap();
        let d = vec![
            NetCDFDimension::new("time".to_string(), 0), // TODO Should this be the length in NoR?
            NetCDFDimension::new("lat".to_string(), 128),
            NetCDFDimension::new("lon".to_string(), 256),
            NetCDFDimension::new("bnds".to_string(), 2),
            NetCDFDimension::new("plev".to_string(), 17),
        ];
        assert_eq!(o, d);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::AttributeList);
        let (i, o) = attribute_list(i).unwrap();
        assert_eq!(o.len(), 18);
        let a = NetCDFAttribute::new(
            "CVS_Id".to_string(),
            NetCDFType::NC_CHAR,
            vec![36, 73, 100, 36],
        );
        assert_eq!(o[0], a);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::VariableList);
        let (i, o) = nelems(i).unwrap();
        assert_eq!(o, 12);
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
