//! Netcdf Head Scratcher - Library for stream parsing netcdf files
use std::io::{Read, Seek, SeekFrom};

use parser::NetCDFHeader;
pub mod constants_and_types;
pub mod error;
pub mod parser;
mod utils;

pub fn update_buffer<F: Seek + Read>(
    header: NetCDFHeader,
    var: String,
    start: Vec<usize>,
    file: &mut F,
    buffer: &mut [u8],
) -> Result<(), std::io::Error> {
    let seek_pos = match header.vars {
        Some(vars) => {
            let v = vars.get(&var).unwrap();
            assert!(v.dims.len() == start.len(), "Lengths are different");
            let offset: usize = start
                .iter()
                .zip(header.seeks.unwrap().get(&var).unwrap())
                .map(|(a, b)| a * b)
                .sum();
            offset as u64 * v.nc_type.extsize() as u64 + v.begin
        }
        None => panic!("Could not find variable '{:?}'", var),
    };
    file.seek(SeekFrom::Start(seek_pos))?;
    file.read_exact(buffer)
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use byteorder::ReadBytesExt;

    fn vec_to_data(buffer: &[u8]) -> Vec<f32> {
        let mut result = vec![0f32; buffer.len() / 4];
        std::io::Cursor::new(buffer)
            .read_f32_into::<byteorder::BigEndian>(&mut result)
            .unwrap();
        result
    }

    #[test]
    fn test_read_netcdf() {
        let nc = include_bytes!("../assets/sresa1b_ncar_ccsm3-example.nc");
        let (_, header) = parser::header(nc).unwrap();
        let path = "assets/sresa1b_ncar_ccsm3-example.nc";
        let mut file = std::fs::File::open(path).unwrap();
        let mut buffer = vec![0u8; 4];
        update_buffer(
            header,
            "tas".to_string(),
            vec![0, 0, 0],
            &mut file,
            &mut buffer,
        )
        .unwrap();
        assert_eq!(vec_to_data(&buffer), vec![215.8935]);
        assert_eq!(buffer, vec![67, 87, 228, 188]);
    }
}
