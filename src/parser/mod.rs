//! Main parsing module
//!
//! # Parser
//! Main parsing module
use crate::error::HeadScratcherError as HSE;
use components::{self as cp, DimensionHM, ListType, NetCDFAttribute, NetCDFVersion, VariableHM};
use cp::NumberOfRecords;
use nom::IResult;

pub mod components;

pub type HSEResult<I, O> = IResult<I, O, HSE<I>>;
// type AttributeHM = HashMap<String, String>;

/// NetCDF file format
#[derive(Debug, PartialEq)]
pub struct NetCDFHeader {
    pub version: NetCDFVersion,
    pub nor: NumberOfRecords,
    pub attrs: Option<Vec<NetCDFAttribute>>,
    pub dims: Option<DimensionHM>,
    pub vars: Option<VariableHM>,
}

impl NetCDFHeader {
    pub fn new(
        version: NetCDFVersion,
        nor: NumberOfRecords,
        attrs: Option<Vec<NetCDFAttribute>>,
        dims: Option<DimensionHM>,
        vars: Option<VariableHM>,
    ) -> Self {
        NetCDFHeader {
            version,
            nor,
            attrs,
            dims,
            vars,
        }
    }
}

pub fn header(i: &[u8]) -> HSEResult<&[u8], NetCDFHeader> {
    // Organisational
    let (i, (_, version, kind)) =
        nom::sequence::tuple((cp::initials, cp::nc_version, cp::number_of_records))(i)?;

    // Dimension list
    let (i, d) = cp::list_type(i)?;
    let (i, dims) = match d {
        ListType::Absent => (i, None),
        ListType::DimensionList => {
            let (i, d) = cp::dimension_list(i)?;
            (i, Some(d))
        }
        _ => Err(nom::Err::Error(HSE::EmptyError))?,
    };

    // Attribute list
    let (i, d) = cp::list_type(i)?;
    let (i, attrs) = match d {
        ListType::Absent => (i, None),
        ListType::AttributeList => {
            let (i, d) = cp::attribute_list(i)?;
            (i, Some(d))
        }
        _ => Err(nom::Err::Error(HSE::EmptyError))?,
    };
    println!("{:?}", version);

    // Variable list
    let (i, d) = cp::list_type(i)?;
    let (i, vars) = match d {
        ListType::Absent => (i, None),
        ListType::VariableList => {
            let (i, d) = cp::variable_list(i, version)?;
            (i, Some(d))
        }
        _ => Err(nom::Err::Error(HSE::EmptyError))?,
    };

    let result = NetCDFHeader::new(version, kind, attrs, dims, vars);

    Ok((i, result))
}

#[cfg(test)]
#[allow(unused_variables, unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn file_example_empty() {
        let i = include_bytes!("../../assets/empty.nc");
        let (i, header) = header(i).unwrap();
    }
    #[test]
    fn file_example_small() {
        let i = include_bytes!("../../assets/small.nc");
        let (i, header) = header(i).unwrap();
    }

    #[test]
    fn file_example_compressible() {
        let i = include_bytes!("../../assets/testrh.nc");
        let (i, header) = header(i).unwrap();
    }

    #[test]
    fn file_nc3_classic() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.nc");
        let (i, header) = header(i).unwrap();
    }

    #[test]
    fn file_nc3_64offset() {
        let i = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.3_nc64.nc");
        let (i, header) = header(i).unwrap();
    }
}
