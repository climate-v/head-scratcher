use headscratcher::NetCDF;

fn main() {
    let filename = "assets/sresa1b_ncar_ccsm3-example.nc".to_string();
    let mut netcdf = NetCDF::new(filename);
    let mapsize = netcdf.mapsize().unwrap();

    // allocate bytes for buffer (mapsize * external size)
    let mut buffer = vec![0u8; mapsize * 4];

    // coordinate for start position
    let coord = vec![0usize,0,0];
    netcdf.update_buffer("tas".to_string(), &coord, &mut buffer).unwrap();
}
