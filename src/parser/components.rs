//! Main parsing module
//!
//! # Parser
//! Main parsing module.
use crate::constants_and_types as csts;
use crate::error::HeadScratcherError as HSE;
use crate::parser::HSEResult;
use nom::{
    bytes::streaming::tag,
    number::{
        complete::{be_f32, be_f64, be_i16, be_i32},
        streaming::{be_u32, be_u64, u8},
    },
};
use std::collections::HashMap;
pub type DimensionHM = HashMap<usize, NetCDFDimension>;
pub type VariableHM = HashMap<String, NetCDFVariable>;
pub type AttributeHM = HashMap<String, NetCDFAttribute>;

/// NetCDF Variable
#[derive(Debug, PartialEq)]
pub struct NetCDFVariable {
    name: String,
    pub dims: Vec<u32>,
    attributes: Option<AttributeHM>,
    pub nc_type: NetCDFType,
    vsize: usize,
    pub begin: u64,
}

impl NetCDFVariable {
    /// Generate a new variable
    pub fn new(
        name: String,
        dims: Vec<u32>,
        attributes: Option<AttributeHM>,
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

    pub fn length(&self) -> usize {
        self.vsize / self.nc_type.extsize()
    }

    pub fn attributes(&self) -> &Option<AttributeHM> {
        &self.attributes
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
pub fn variable_list(i: &[u8], version: NetCDFVersion) -> HSEResult<&[u8], VariableHM> {
    let (mut i, mut count) = nelems(i)?;
    let mut result = HashMap::new();
    while count > 0 {
        let (k, v) = variable(i, version)?;
        result.insert(v.name.clone(), v); // TODO: Implement without cloning
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
    data: NetCDFTypeInstance,
}

impl NetCDFAttribute {
    /// Create a new NetCDF Attribute
    pub fn new(name: String, nc_type: NetCDFType, data: Vec<u8>) -> Self {
        let value = match nc_type {
            NetCDFType::NC_CHAR => {
                let text = std::str::from_utf8(&data[..]).unwrap();
                NetCDFTypeInstance::STRING(text.to_string())
            }
            NetCDFType::NC_FLOAT => {
                let (_, v) = float(data.as_slice()).unwrap();
                NetCDFTypeInstance::FLOAT(v)
            }
            NetCDFType::NC_DOUBLE => {
                let (_, v) = double(data.as_slice()).unwrap();
                NetCDFTypeInstance::DOUBLE(v)
            }
            NetCDFType::NC_INT => {
                let (_, v) = integer(data.as_slice()).unwrap();
                NetCDFTypeInstance::INT(v)
            }
            NetCDFType::NC_SHORT => {
                let (_, v) = short(data.as_slice()).unwrap();
                NetCDFTypeInstance::SHORT(v)
            }
            NetCDFType::NC_BYTE => NetCDFTypeInstance::_RAW(data),
        };
        NetCDFAttribute {
            name,
            nc_type,
            data: value,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match &self.data {
            NetCDFTypeInstance::STRING(content) => Some(content.clone()),
            _ => None,
        }
    }
}

/// Parse a single NetCDF attribute [combined]
pub fn attribute(i: &[u8]) -> HSEResult<&[u8], NetCDFAttribute> {
    let (i, (name, nc_type, nelems)) = nom::sequence::tuple((name, nc_type, nelems))(i)?;
    let (i, data) = nom::bytes::streaming::take(nc_type.extsize() * nelems as usize)(i)?;
    // names are padded to the next 4-byte boundary
    // println!("{:?} {:?} {:?} {:?}", name, nc_type, nelems, data);
    let drop = padding(nc_type.extsize() as u32 * nelems);
    let (i, _) = nom::bytes::streaming::take(drop as u8)(i)?;
    let result = NetCDFAttribute::new(name.to_string(), nc_type, data.to_vec());
    // println!("{:?}", result);
    Ok((i, result))
}

/// Parse a list of NetCDF attributes [combined]
pub fn attribute_list(i: &[u8]) -> HSEResult<&[u8], AttributeHM> {
    let (i, attrs) = nom::multi::length_count(nelems, attribute)(i)?;
    let mut result: AttributeHM = HashMap::new();
    for a in attrs.into_iter() {
        result.insert(a.name.clone(), a);
    }
    Ok((i, result))
}

/// NetCDF Dimension
#[derive(Debug, PartialEq)]
pub struct NetCDFDimension {
    pub(crate) name: String,
    pub length: usize,
}

impl NetCDFDimension {
    /// Create new NetCDF dimension
    pub fn new(name: String, length: usize) -> Self {
        NetCDFDimension { name, length }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

/// Parse a single NetCDF dimension [combined]
pub fn dimension(i: &[u8]) -> HSEResult<&[u8], NetCDFDimension> {
    let (i, (name, dim_length)) = nom::sequence::tuple((name, dim_length))(i)?;
    let ncdim = NetCDFDimension::new(name.to_string(), dim_length as usize);
    Ok((i, ncdim))
}

/// Parse a list of NetCDF dimensions [combined]
pub fn dimension_list(i: &[u8]) -> HSEResult<&[u8], DimensionHM> {
    let (i, dims) = nom::multi::length_count(nelems, dimension)(i)?;
    let mut result: DimensionHM = HashMap::new();
    for (i, d) in dims.into_iter().enumerate() {
        result.insert(i, d);
    }
    Ok((i, result))
}

/// NetCDF attribute value
#[derive(Debug, PartialEq)]
pub enum NetCDFTypeInstance {
    STRING(String),
    CHAR(u8),
    SHORT(i16),
    INT(i32),
    FLOAT(f32),
    DOUBLE(f64),
    _RAW(Vec<u8>),
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

/// Parse float [atomic]
pub fn float(i: &[u8]) -> HSEResult<&[u8], f32> {
    be_f32(i)
}

/// Parse integer [atomic]
pub fn integer(i: &[u8]) -> HSEResult<&[u8], i32> {
    be_i32(i)
}

/// Parse short [atomic]
pub fn short(i: &[u8]) -> HSEResult<&[u8], i16> {
    be_i16(i)
}

/// Parse double [atomic]
pub fn double(i: &[u8]) -> HSEResult<&[u8], f64> {
    be_f64(i)
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
        csts::ZERO => Ok((i, true)),
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

pub fn eof(i: &[u8]) -> HSEResult<&[u8], bool> {
    let (i, _) = nom::combinator::eof(i)?;
    Ok((i, true))
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;
    use core::panic;
    use std::{fs::File, io::BufReader, io::Read};

    #[test]
    fn file_example_empty() {
        let file = File::open("assets/empty.nc").unwrap();
        let mut reader = BufReader::new(file);
        let mut i = Vec::new();
        reader.read_to_end(&mut i).unwrap();
        let (i, o) = initials(&i[..]).unwrap();
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
        let (i, o) = eof(i).unwrap();
        assert!(o); // EOF
    }

    #[test]
    fn file_example_small() {
        let file = File::open("assets/small.nc").unwrap();
        let mut reader = BufReader::new(file);
        let mut i = Vec::new();
        reader.read_to_end(&mut i).unwrap();
        let (i, o) = initials(&i[..]).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(0));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = dimension_list(i).unwrap();
        let d = vec![NetCDFDimension::new("dim".to_string(), 5)];
        assert_eq!(o[&0], d[0]);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::Absent);
    }

    #[test]
    fn file_example_compressible() {
        let file = File::open("assets/testrh.nc").unwrap();
        let mut reader = BufReader::new(file);
        let mut i = Vec::new();
        reader.read_to_end(&mut i).unwrap();
        let (i, o) = initials(&i[..]).unwrap();
        assert_eq!(o, b"CDF");
        let (i, o) = nc_version(i).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (i, o) = number_of_records(i).unwrap();
        assert_eq!(o, NumberOfRecords::NonNegative(0));
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::DimensionList);
        let (i, o) = dimension_list(i).unwrap();
        let d = vec![NetCDFDimension::new("dim1".to_string(), 10_000)];
        assert_eq!(o[&0], d[0]);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::Absent);
    }

    #[test]
    fn file_nc3_classic() {
        let file = File::open("assets/sresa1b_ncar_ccsm3-example.nc").unwrap();
        let mut reader = BufReader::new(file);
        let mut i = Vec::new();
        reader.read_to_end(&mut i).unwrap();
        let (i, o) = initials(&i[..]).unwrap();
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
        for i in 0..5 {
            assert_eq!(o[&i], d[i]);
        }
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::AttributeList);
        let (i, o) = attribute_list(i).unwrap();
        assert_eq!(o.len(), 18);
        let a = NetCDFAttribute::new(
            "CVS_Id".to_string(),
            NetCDFType::NC_CHAR,
            vec![36, 73, 100, 36],
        );
        assert_eq!(o["CVS_Id"], a);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::VariableList);
        let (i, o) = variable_list(i, v).unwrap();
        assert_eq!(o["area"].name, "area");
        assert_eq!(o["area"].dims, vec![0, 1]);
        assert_eq!(o["area"].begin, 7564);
        assert_eq!(o["area"].vsize, 131072);
        assert_eq!(o["area"].nc_type, NetCDFType::NC_FLOAT);
        // TODO check other variables
    }

    #[test]
    fn file_nc3_64offset() {
        let file = File::open("assets/sresa1b_ncar_ccsm3-example.3_nc64.nc").unwrap();
        let mut reader = BufReader::new(file);
        let mut i = Vec::new();
        reader.read_to_end(&mut i).unwrap();
        let (i, o) = initials(&i[..]).unwrap();
        assert_eq!(o, b"CDF");
        let (i, v) = nc_version(i).unwrap();
        assert_eq!(v, NetCDFVersion::Offset64);
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
        for i in 0..5 {
            assert_eq!(o[&i], d[i]);
        }
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::AttributeList);
        let (i, o) = attribute_list(i).unwrap();
        assert_eq!(o.len(), 18);
        let a = NetCDFAttribute::new(
            "CVS_Id".to_string(),
            NetCDFType::NC_CHAR,
            vec![36, 73, 100, 36],
        );
        assert_eq!(o["CVS_Id"], a);
        let (i, o) = list_type(i).unwrap();
        assert_eq!(o, ListType::VariableList);
        let (i, o) = variable_list(i, v).unwrap();
        // TODO: Read about fill values
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
        let file = File::open("assets/test_hgroups.nc").unwrap();
        let mut reader = BufReader::new(file);
        let mut i = Vec::new();
        reader.read_to_end(&mut i).unwrap();
        magic(&i[..]).unwrap();
    }

    #[test]
    fn test_wrong_version() {
        let file = File::open("assets/test_hgroups.nc").unwrap();
        let mut reader = BufReader::new(file);
        let mut i = Vec::new();
        reader.read_to_end(&mut i).unwrap();
        let e = nc_version(&i[..]).unwrap_err();
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
