//! Netcdf Head Scratcher - Library for stream parsing netcdf files
use error::HeadScratcherError;
use parser::NetCDFHeader;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use utils::calc_seek;

#[rustfmt::skip]
pub mod constants_and_types;
pub mod error;
pub mod parser;
mod utils;

#[derive(Debug, PartialEq)]
pub struct NetCDF<F: Seek + Read> {
    file: F,
    header: NetCDFHeader,
}

impl<F: Seek + Read> NetCDF<F> {
    pub fn new_from_file(mut file: F) -> Self {
        let h = NetCDFHeader::from_file(&mut file).unwrap();
        NetCDF { file, header: h }
    }

    pub fn update_buffer(
        &mut self,
        variable: String,
        start: &[usize],
        buffer: &mut [u8],
    ) -> Result<(), HeadScratcherError<String>> {
        let seek_pos = match (&self.header.vars, &self.header.seeks) {
            (Some(v), Some(s)) => calc_seek(v, s, variable.clone(), start),
            (_, _) => return Err(HeadScratcherError::NoVariablesInFile),
        };
        let pos = match seek_pos {
            Some(k) => k,
            None => return Err(HeadScratcherError::VariableNotFound(variable)),
        };
        self.file.seek(SeekFrom::Start(pos))?;
        let ok = self.file.read_exact(buffer)?;
        Ok(ok)
    }

    pub fn header(&self) -> &NetCDFHeader {
        &self.header
    }

    pub fn mapsize(&self) -> Result<usize, HeadScratcherError<String>> {
        match &self.header.dims {
            Some(dims) => {
                let lon = crate::utils::get_coordinate_dim_id(
                    &dims,
                    crate::constants_and_types::LONGITUDE_CANDIDATES,
                );
                let lat = crate::utils::get_coordinate_dim_id(
                    &dims,
                    crate::constants_and_types::LATITUDE_CANDIDATES,
                );
                match (lon, lat) {
                    (Ok(lon_id), Ok(lat_id)) => {
                        return Ok(
                            dims.get(&lon_id).unwrap().length * dims.get(&lat_id).unwrap().length
                        )
                    }
                    (Ok(_), Err(_)) => {
                        return Err(HeadScratcherError::CouldNotFindDimension(
                            "Latitude".to_string(),
                        ))
                    }
                    (Err(_), Ok(_)) => {
                        return Err(HeadScratcherError::CouldNotFindDimension(
                            "Longitude".to_string(),
                        ))
                    }
                    (Err(_), Err(_)) => {
                        return Err(HeadScratcherError::CouldNotFindDimension(
                            "Longitude and Latitude".to_string(),
                        ))
                    }
                };
            }
            _ => Err(HeadScratcherError::NoDimensionsInFile),
        }
    }
}

impl NetCDF<File> {
    pub fn new(filename: String) -> Self {
        let mut fs = File::open(&filename).unwrap();
        let h = NetCDFHeader::from_file(&mut fs).unwrap();
        NetCDF {
            file: fs,
            header: h,
        }
    }
}

#[cfg(features = "border")]
use byteorder::ReadBytesExt;

#[cfg(features = "border")]
pub fn vec_to_data(buffer: &[u8]) -> Vec<f32> {
    let mut result = vec![0f32; buffer.len() / 4];
    std::io::Cursor::new(buffer)
        .read_f32_into::<byteorder::BigEndian>(&mut result)
        .unwrap();
    result
}

#[cfg(not(features = "border"))]
pub fn vec_to_data(buffer: &[u8]) -> Vec<f32> {
    let (_, result) =
        nom::multi::count(crate::parser::components::float, buffer.len() / 4)(buffer).unwrap();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_netcdf() {
        let filename = "assets/sresa1b_ncar_ccsm3-example.nc".to_string();
        let mut netcdf = NetCDF::new(filename);
        let mut buffer = vec![0u8; 4];
        netcdf
            .update_buffer("tas".to_string(), &vec![0, 0, 0], &mut buffer)
            .unwrap();
        assert_eq!(vec_to_data(&buffer), vec![215.8935]);
        assert_eq!(buffer, vec![67, 87, 228, 188]);
        let map = netcdf.mapsize().unwrap();
        assert_eq!(map, 256 * 128)
    }
}
