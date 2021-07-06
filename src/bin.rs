use headscratcher::parser::header;
use clap::{App, Arg, crate_authors, crate_version};

fn main() {
    let libv = crate_version!();
    let binv = "0.1.1";
    let v = format!("\nlib v{} \nbin v{}", libv, binv);
    let matches = App::new("NetCDF Head Scratcher")
                    .version(v.as_str())
                    .author(crate_authors!())
                    .about("I/O support for netCDF files")
                    .arg(Arg::with_name("INPUT")
                        .help("Sets the input file to use")
                        .required(true)
                        .index(1))
                    .arg(Arg::with_name("dimensions")
                        .short("d")
                        .long("dimension")
                        .value_name("DIMENSION")
                        .takes_value(true)
                        .multiple(true)
                        .help("Print all information about a dimension (i.e. coordinate variabele)"))
                    .arg(Arg::with_name("variables")
                        .short("v")
                        .long("variable")
                        .value_name("VARIABLE")
                        .takes_value(true)
                        .multiple(true)
                        .help("Print all information about a data variable"))
                    .get_matches();

    let i = include_bytes!("../assets/sresa1b_ncar_ccsm3-example.nc"); // TODO: Read actual file
    let (_, h) = header(i).unwrap();
    let vars = h.vars.unwrap();
    for v in matches.values_of("variables").unwrap() {
        println!("{:#?}", &vars[v]);
    }
    // TODO: Print coordinate variable requests
    // println!("Dimensions: {:#?}", h.dims);
}
