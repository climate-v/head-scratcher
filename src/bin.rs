use std::io::Read;

use headscratcher::parser::header;
use clap::{App, Arg, crate_authors, crate_version};

fn main() -> std::io::Result<()> {
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

    let mut file = std::fs::File::open(matches.value_of("INPUT").unwrap())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let (_, h) = header(buffer.as_slice()).unwrap();
    let vars = h.vars.unwrap();
    for v in matches.values_of("variables").unwrap() {
        println!("{:#?}", &vars[v]);
    }
    Ok(())
}
