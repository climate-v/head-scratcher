//! Main parsing module
//!
//! # Parser
//! Main parsing module
use crate::error::HeadScratcherError as HSE;
use components::{NetCDFDimension, NetCDFType, NetCDFVariable, NetCDFVersion};
use nom::IResult;
use std::collections::HashMap;

pub mod components;

pub type HSEResult<I, O> = IResult<I, O, HSE<I>>;
pub type AttributeHM = HashMap<String, String>;
pub type DimensionHM = HashMap<u32, NetCDFDimension>;
pub type VariableHM = HashMap<String, NetCDFVariable>;

/// NetCDF file format
#[derive(Debug)]
pub struct NetCDFHeader {
    version: NetCDFVersion,
    kind: NetCDFType,
    attrs: Option<AttributeHM>,
    dims: Option<DimensionHM>,
    vars: Option<VariableHM>,
}
