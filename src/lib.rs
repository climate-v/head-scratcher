//! Netcdf Head Scratcher - Library for stream parsing netcdf files
use std::io::{Read, Seek, SeekFrom};

use parser::NetCDFHeader;
pub mod constants_and_types;
pub mod error;
pub mod parser;

// https://cluster.earlham.edu/bccd-ng/testing/mobeen/GALAXSEEHPC/netcdf-4.1.3/man4/netcdf.html#Computing-Offsets
fn product_vector(vecs: &[usize], record: bool) -> Vec<usize> {
    let mut prod = 1usize;
    let mut result: Vec<usize> = Vec::new();
    for v in vecs.iter().rev() {
        prod *= v;
        result.insert(0, prod);
    }
    result
}

fn read_first_f32<S: Seek + Read>(
    header: NetCDFHeader,
    var: String,
    time: Option<usize>,
    lev: Option<usize>,
    file: &mut S,
) -> f32 {
    let vars = header.vars.unwrap();
    let pos = vars[&var].begin;
    file.seek(SeekFrom::Start(pos)).unwrap();

    let mut contents = vec![0u8; 4];
    file.read_exact(&mut contents).unwrap();
    let (_, val) = parser::components::float(&contents).unwrap();
    val
}

fn update_buffer<F: Seek + Read>(
    header: NetCDFHeader,
    var: String,
    time: Option<usize>,
    lev: Option<usize>,
    file: &mut F,
    buffer: &mut [u8],
) -> Result<(), std::io::Error> {
    let vars = header.vars.unwrap();
    let seek_pos = vars[&var].begin;
    let time_idx = time.unwrap_or(1);
    let lev_idx = lev.unwrap_or(1);
    file.seek(SeekFrom::Start(seek_pos))?;
    file.read_exact(buffer)
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_read_netcdf() {
        let nc = include_bytes!("../assets/sresa1b_ncar_ccsm3-example.nc");
        let (_, header) = parser::header(nc).unwrap();
        let path = "assets/sresa1b_ncar_ccsm3-example.nc";
        let mut file = std::fs::File::open(path).unwrap();
        let val = read_first_f32(header, "tas".to_string(), None, None, &mut file);
        assert_eq!(val, 215.8935)
    }
    //     let var = &file.variable("tas").unwrap();
    //     let first: f32 = var.value(None).unwrap();
    //     assert_eq!(first, 215.8935);
    //     let flevel = var
    //         .values::<f32>(Some(&[0, 0, 0]), Some(&[1, 1, 5]))
    //         .unwrap();
    //     let expected = vec![215.8935, 215.80531, 215.73935, 215.66304, 215.61963];
    //     assert_eq!(flevel.into_raw_vec(), expected)
    // }

    #[test]
    fn test_read_level_netcdf() {
        let nc = include_bytes!("../assets/sresa1b_ncar_ccsm3-example.nc");
        let (_, header) = parser::header(nc).unwrap();
        let path = "assets/sresa1b_ncar_ccsm3-example.nc";
        let mut file = std::fs::File::open(path).unwrap();
        let mut buffer = vec![0u8; 4];
        update_buffer(
            header,
            "tas".to_string(),
            None,
            None,
            &mut file,
            &mut buffer,
        )
        .unwrap();

        assert_eq!(buffer, vec![67, 87, 228, 188])
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
