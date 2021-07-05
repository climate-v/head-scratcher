use netcdf_head_scratcher::parser::header;

fn main() {
    let i = include_bytes!("../assets/sresa1b_ncar_ccsm3-example.nc");
    let (_, h) = header(i).unwrap();
    println!("Variables: {:#?}", h.vars);
    println!("Dimensions: {:#?}", h.dims);
}
