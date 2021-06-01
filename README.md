# head-scratcher
Library for reading the header information from netcdf files.

## Use case
This library allows to read a netcdf file as a raw byte stream and extract meta information.
This meta information is then used to optimize memory allocation and usage.

## Extracted information
- [ ] List of variables
- [ ] List of coordinate variables
- [ ] For each variable
  - [ ] Name
  - [ ] Size
  - [ ] Offset
  - [ ] Datatype
  - [ ] List of coordinates
  - [ ] is_coordinate

## Resources
- [Netcdf specification (incl. BNF)](https://cluster.earlham.edu/bccd-ng/testing/mobeen/GALAXSEEHPC/netcdf-4.1.3/man4/netcdf.html#File-Format)
- [Official Tutorial for nom (possibly up-to-date)](https://github.com/Geal/nom/tree/master/doc)
- [Tutorial for nom (possibly old)](https://blog.logrocket.com/parsing-in-rust-with-nom/)
