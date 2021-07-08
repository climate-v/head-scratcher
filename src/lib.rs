//! Netcdf Head Scratcher - Library for stream parsing netcdf files
pub mod constants_and_types;
mod error;
pub mod parser;
mod utils;

#[cfg(test)]
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
        header
            .update_buffer(&"tas".to_string(), &vec![0, 0, 0], &mut file, &mut buffer)
            .unwrap();
        assert_eq!(vec_to_data(&buffer), vec![215.8935]);
        assert_eq!(buffer, vec![67, 87, 228, 188]);
    }
}
