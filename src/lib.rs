//! Netcdf Head Scratcher - Library for stream parsing netcdf files
pub mod constants_and_types;
pub mod error;
pub mod parser;

fn product_vector(vecs: &[usize], record: bool) -> Vec<usize> {
    unimplemented!()
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    // #[test]
    // fn test_read_netcdf() {
    //     let path = "assets/sresa1b_ncar_ccsm3-example.nc";
    //     let file = netcdf::open(path).unwrap();
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
