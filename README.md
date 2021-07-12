# head-scratcher
Library for reading the header information from netcdf files.

## Use case
This library allows to read a netcdf file as a raw byte stream and extract meta information.
This meta information is then used to optimize memory allocation and memory usage.

## Example

```rust
use headscratcher::NetCDF;
use headscratcher::error::HeadScratcherError;

fn main() -> Result<(), HeadScratcherError<String>>{
    let filename = "assets/sresa1b_ncar_ccsm3-example.nc".to_string();
    let mut netcdf = NetCDF::new(filename);
    let mapsize = netcdf.mapsize()?;

    // allocate bytes for buffer (mapsize * external size)
    let mut buffer = vec![0u8; mapsize * 4];

    // coordinate for start position
    let coord = vec![0usize,0,0];
    netcdf.update_buffer("tas".to_string(), &coord, &mut buffer)?;
    Ok(())
}

```


## Resources
- [Netcdf specification (incl. BNF)](https://cluster.earlham.edu/bccd-ng/testing/mobeen/GALAXSEEHPC/netcdf-4.1.3/man4/netcdf.html#File-Format)
- [Official Tutorial for nom (possibly up-to-date)](https://github.com/Geal/nom/tree/master/doc)
- [Tutorial for nom (possibly old)](https://blog.logrocket.com/parsing-in-rust-with-nom/)
