extern crate yaml_rust;

use std::fs::File;
use std::fs::PathExt;  // for .exists()
use std::path::Path;
use std::io::Read;
use std::io::BufReader;
use yaml_rust::YamlLoader;
use yaml_rust::Yaml;

pub fn read_file(filename: &str) -> String {
    if !Path::new(filename).exists() {
        panic!(format!("File {} does not exist", filename));
    }
    let file = File::open(filename).unwrap();
    let mut buf = BufReader::new(&file);

    let mut s = String::new();
    buf.read_to_string(&mut s);
    s
}

pub fn load_yaml_file(filename: &str) -> Yaml {
    let s = read_file(filename);
    match YamlLoader::load_from_str(s.as_str()) {
        Ok(yaml_docs) => { yaml_docs[0].clone() }
        Err(err) => { panic!(format!("Failed to load yaml file {}\n{:?}", filename, err)); }
    }
}

