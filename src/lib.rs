//! Netcdf Head Scratcher - Library for stream parsing netcdf files
use std::io::{Read, Seek, SeekFrom};

use parser::NetCDFHeader;
pub mod constants_and_types;
pub mod error;
pub mod parser;

fn product_vector(vecs: &[usize], _record: bool) -> Vec<usize> {
    // https://cluster.earlham.edu/bccd-ng/testing/mobeen/GALAXSEEHPC/netcdf-4.1.3/man4/netcdf.html#Computing-Offsets
    let mut prod = 1usize;
    let mut result: Vec<usize> = Vec::new();
    for v in vecs.iter().rev() {
        prod *= v;
        result.insert(0, prod);
    }
    result
}

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

    #[test]
    fn test_product_vector() {
        let vecs: Vec<usize> = vec![5, 3, 2, 7];
        let record = false;
        let expected: Vec<usize> = vec![210, 42, 14, 7];
        let result = product_vector(&vecs, record);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_rec_product_vector() {
        let vecs: Vec<usize> = vec![0, 2, 9, 4];
        let record = true;
        let expected: Vec<usize> = vec![0, 72, 36, 4];
        let result = product_vector(&vecs, record);
        assert_eq!(result, expected);
    }
}
