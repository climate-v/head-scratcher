use crate::{
    error::HeadScratcherError,
    parser::components::DimensionHM,
    parser::{components::VariableHM, SeeksHM},
};

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

pub(crate) fn calc_seek(v: &VariableHM, s: &SeeksHM, name: String, start: &[usize]) -> Option<u64> {
    match (v.get(&name), s.get(&name)) {
        (Some(va), Some(se)) => {
            assert!(va.dims.len() == start.len(), "Lengths are different");
            let offset: usize = start.iter().zip(se).map(|(a, b)| a * b).sum();
            let result = offset as u64 * va.nc_type.extsize() as u64 + va.begin;
            Some(result)
        }
        (_, _) => None,
    }
}

pub(crate) fn get_coordinate_dim_id(
    dims: &DimensionHM,
    candidates: &[&str],
) -> Result<usize, HeadScratcherError<String>> {
    for (k, d) in dims.iter() {
        if candidates.contains(&&*d.name) {
            return Ok(*k);
        } else {
            continue;
        }
    }
    let msg = format!("Candidate space: {:?}", candidates);
    Err(HeadScratcherError::CouldNotFindDimension(msg))
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
