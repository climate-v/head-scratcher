//! Main parsing module
//!
//! # Parser
//! Main parsing module
use crate::error::HeadScratcherError as HSE;
use crate::utils::{calc_seek, product_vector};
use components::{
    self as cp, AttributeHM, DimensionHM, ListType, NetCDFVersion, NumberOfRecords, VariableHM,
};
use nom::IResult;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
pub mod components;
pub type HSEResult<I, O> = IResult<I, O, HSE<I>>;
pub type SeeksHM = HashMap<String, Vec<usize>>;
const BUFFER: usize = 4096; // bytes

/// NetCDF file format
#[derive(Debug, PartialEq)]
pub struct NetCDFHeader {
    pub version: NetCDFVersion,
    pub nor: NumberOfRecords,
    pub attrs: Option<AttributeHM>,
    pub dims: Option<DimensionHM>,
    pub vars: Option<VariableHM>,
    pub seeks: Option<SeeksHM>,
}

impl NetCDFHeader {
    pub fn new(
        version: NetCDFVersion,
        nor: NumberOfRecords,
        attrs: Option<AttributeHM>,
        dims: Option<DimensionHM>,
        vars: Option<VariableHM>,
        seeks: Option<SeeksHM>,
    ) -> Self {
        NetCDFHeader {
            version,
            nor,
            attrs,
            dims,
            vars,
            seeks,
        }
    }
    pub fn update_buffer<F: Seek + Read>(
        &self,
        var: String,
        start: &[usize],
        file: &mut F,
        buffer: &mut [u8],
    ) -> Result<(), std::io::Error> {
        let seek_pos = match (&self.vars, &self.seeks) {
            (Some(v), Some(s)) => calc_seek(v, s, var, start),
            (_, _) => None,
        };
        file.seek(SeekFrom::Start(seek_pos.unwrap()))?;
        file.read_exact(buffer)
    }

    pub fn from_file<F: Read>(file: &mut F) -> Result<NetCDFHeader, HSE<String>> {
        let mut buf: Vec<u8> = vec![0; BUFFER];
        let mut head: Vec<u8> = Vec::new();
        loop {
            let count = file.read(&mut buf)?;
            head.append(&mut buf[..count].to_vec());
            match header(&head) {
                Ok((_, h)) => return Ok(h),
                Err(nom::Err::Incomplete(nom::Needed::Size(_))) => continue,
                Err(nom::Err::Failure(err)) => {
                    return match err {
                        HSE::NomError(_, _) => Err(HSE::InvalidFile),
                        e => Err(e.cast().unwrap()) // cannot be None as the only case this can happen is covered above
                    }
                },
                Err(nom::Err::Error(err)) => {
                    return match err {
                        HSE::NomError(_, _) => Err(HSE::InvalidFile),
                        e => Err(e.cast().unwrap()) // cannot be None as the only case this can happen is covered above
                    }
                },
                Err(err) => panic!("Other error: {:?}", err),
            }
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

    // Seek calculation
    let seeks = calculate_seeks(&vars, &dims);
    let result = NetCDFHeader::new(version, kind, attrs, dims, vars, seeks);

    Ok((i, result))
}

fn calculate_seeks(vars: &Option<VariableHM>, dims: &Option<DimensionHM>) -> Option<SeeksHM> {
    match (vars, dims) {
        (Some(v), Some(d)) => Some(clc(v, d)),
        (_, _) => None,
    }
}

fn clc(vars: &VariableHM, dims: &DimensionHM) -> SeeksHM {
    let mut result: HashMap<String, Vec<usize>> = HashMap::new();
    for (k, v) in vars.iter() {
        let mut seeks: Vec<usize> = Vec::new();
        for d in v.dims.iter() {
            seeks.push(dims[&(*d as usize)].length)
        }
        let mut seeks = product_vector(&seeks, false);
        seeks.push(1);
        seeks.remove(0);
        result.insert(k.clone(), seeks);
    }
    result
}

#[cfg(test)]
#[allow(unused_variables, unused_imports)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn file_example_empty() {
        let filename = "assets/empty.nc".to_string();
        let mut file = File::open(filename).unwrap();
        let h = NetCDFHeader::from_file(&mut file).unwrap();
    }
    #[test]
    fn file_example_small() {
        let filename = "assets/small.nc".to_string();
        let mut file = File::open(filename).unwrap();
        let h = NetCDFHeader::from_file(&mut file).unwrap();
    }

    #[test]
    fn file_example_compressible() {
        let filename = "assets/testrh.nc".to_string();
        let mut file = File::open(filename).unwrap();
        let h = NetCDFHeader::from_file(&mut file).unwrap();
    }

    #[test]
    fn file_nc3_classic() {
        let filename = "assets/sresa1b_ncar_ccsm3-example.nc".to_string();
        let mut file = File::open(filename).unwrap();
        let h = NetCDFHeader::from_file(&mut file).unwrap();
    }
    #[test]
    fn file_nc3_64offset() {
        let filename = "assets/sresa1b_ncar_ccsm3-example.3_nc64.nc".to_string();
        let mut file = File::open(filename).unwrap();
        let h = NetCDFHeader::from_file(&mut file).unwrap();
    }

    #[test]
    fn test_seeks() {
        let filename = "assets/sresa1b_ncar_ccsm3-example.nc".to_string();
        let mut file = File::open(filename).unwrap();
        let header = NetCDFHeader::from_file(&mut file).unwrap();
        let seeks = calculate_seeks(&header.vars, &header.dims).unwrap();
        let expected = vec![
            ("plev".to_string(), vec![1usize]),
            ("lon".to_string(), vec![1]),
            ("lat".to_string(), vec![1]),
            ("time".to_string(), vec![1]),
            ("area".to_string(), vec![256, 1]),
            ("msk_rgn".to_string(), vec![256, 1]),
            ("pr".to_string(), vec![32768, 256, 1]),
            ("tas".to_string(), vec![32768, 256, 1]),
            ("ua".to_string(), vec![557056, 32768, 256, 1]),
        ];
        for (v, e) in expected.iter() {
            println!("{:?}", v);
            let calculated = &seeks[v];
            assert_eq!(calculated, e)
        }
    }
}
