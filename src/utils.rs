
pub(crate) fn product_vector(vecs: &[usize], _record: bool) -> Vec<usize> {
    // https://cluster.earlham.edu/bccd-ng/testing/mobeen/GALAXSEEHPC/netcdf-4.1.3/man4/netcdf.html#Computing-Offsets
    let mut prod = 1usize;
    let mut result: Vec<usize> = Vec::new();
    for v in vecs.iter().rev() {
        prod *= v;
        result.insert(0, prod);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
