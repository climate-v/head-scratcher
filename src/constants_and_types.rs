//! Constants and Types
#![allow(non_camel_case_types, unused_attributes)]
#![rustfmt::skip]

// Constants
pub const STREAMING:    u32 = 0xFF_FF_FF_FF;
pub const ZERO:         u32 = 0x00_00_00_00;

pub const NC_DIMENSION: u32 = 0x00_00_00_0A;
pub const NC_VARIABLE:  u32 = 0x00_00_00_0B;
pub const NC_ATTRIBUTE: u32 = 0x00_00_00_0C;

pub const NC_BYTE:      u32 = 0x00_00_00_01;
pub const NC_CHAR:      u32 = 0x00_00_00_02;
pub const NC_SHORT:     u32 = 0x00_00_00_03;
pub const NC_INT:       u32 = 0x00_00_00_04;
pub const NC_FLOAT:     u32 = 0x00_00_00_05;
pub const NC_DOUBLE:    u32 = 0x00_00_00_06;

// Types
pub type NON_NEG = u32;
