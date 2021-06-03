use nom::{IResult, bytes::streaming::tag, number::streaming::u8};
use crate::error::HeadScratcherError as HSE;

#[derive(Debug, PartialEq)]
pub enum NetCDFVersion {
    Classic,
    Offset64,
}

/// Get a single byte
fn take_u8(i: &[u8]) -> IResult<&[u8], u8, HSE<&[u8]>> {u8(i)}

/// Check file initials [atomic]
pub fn initials(i: &[u8]) -> IResult<&[u8], &[u8], HSE<&[u8]>> {tag("CDF")(i)}

/// Check NetCDF version [atomic]
pub fn nc_version(i: &[u8]) -> IResult<&[u8], NetCDFVersion, HSE<&[u8]>> {
    let (i, o) = take_u8(i)?;
    match o {
        1 => Ok((i, NetCDFVersion::Classic)),
        2 => Ok((i, NetCDFVersion::Offset64)),
        _ => Err(nom::Err::Error(HSE::UnsupportedNetCDFVersion))
    }
}

/// Check NetCDF magic bytes [combined]
pub fn magic(i: &[u8]) -> IResult<&[u8], NetCDFVersion, HSE<&[u8]>> {
    let (i, _ ) = initials(i)?;
    let (i, v) = nc_version(i)?;
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;

    #[test]
    fn test_magic_nc() {
        let f = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.nc");
        let (_, v) = magic(f).unwrap();
        assert_eq!(v, NetCDFVersion::Classic);
        let f = include_bytes!("../../assets/testrh.nc");
        let (_, v) = magic(f).unwrap();
        assert_eq!(v, NetCDFVersion::Classic);
    }


    #[test]
    fn test_initials_nc() {
        let f = include_bytes!("../../assets/sresa1b_ncar_ccsm3-example.nc");
        initials(f).unwrap();
        let f = include_bytes!("../../assets/testrh.nc");
        initials(f).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_initials_hdf() {
        let f = include_bytes!("../../assets/test_hgroups.nc");
        magic(f).unwrap();
    }

    #[test]
    fn test_wrong_version() {
        let f = include_bytes!("../../assets/test_hgroups.nc");
        let e = nc_version(f).unwrap_err();
        match e {
            nom::Err::Error(e) => assert_eq!(e, HSE::UnsupportedNetCDFVersion),
            _ => panic!("Unexpected error {:?}", e)
        }
    }

    #[test]
    fn test_nc_version() {
        let (_, o) = nc_version(&[1u8]).unwrap();
        assert_eq!(o, NetCDFVersion::Classic);
        let (_, o) = nc_version(&[2u8]).unwrap();
        assert_eq!(o, NetCDFVersion::Offset64);
    }
}
