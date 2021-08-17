//! Constants and types
#![allow(non_camel_case_types, unused_attributes)]

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

pub const FILL_BYTE:    u8 = 0x81;
pub const FILL_CHAR:    u8 = 0x00;
pub const FILL_SHORT:  u16 = 0x80_01;
pub const FILL_INT:    u32 = 0x80_00_00_01;
pub const FILL_FLOAT:  u32 = 0x7C_F0_00_00;
pub const FILL_DOUBLE: u64 = 0x47_9E_00_00_00_00;

// Types
pub type NON_NEG = u32;

// Naming standards
pub const LONGITUDE_CANDIDATES: &[&str] = &["lon", "longitude"];
pub const LATITUDE_CANDIDATES:  &[&str] = &["lat", "latitude"];
pub const NCELLS_CANDIDATES:    &[&str] = &["ncells"];
pub const ALTITUDE_CANDIDATES:  &[&str] = &["lev", "level", "alt", "height"];
pub const TIME_CANDIDATES:      &[&str] = &["time"];
