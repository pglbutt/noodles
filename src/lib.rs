#![feature(fs_walk)]
#![feature(path_relative_from)]
#![feature(convert)]
#![feature(debug_builders)]
#![feature(plugin)]
#![feature(path_ext)]
#![plugin(docopt_macros)]

extern crate curl;
extern crate docopt;
extern crate rustc_serialize;
extern crate yaml_rust;

pub mod spag;
