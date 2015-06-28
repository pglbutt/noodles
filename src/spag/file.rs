extern crate yaml_rust;

use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::io::BufReader;

pub fn read_file(filename: &str) -> Result<String, String> {
    if !Path::new(filename).exists() {
        return Err(format!("File {} does not exist", filename));
    }
    let file = File::open(filename).unwrap();
    let mut buf = BufReader::new(&file);

    let mut s = String::new();
    buf.read_to_string(&mut s).unwrap();
    Ok(s)
}

pub fn write_file(filename: &str, contents: &str) {
    let mut f = File::create(filename).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
}

pub fn ensure_dir_exists(dir: &str) {
    let path = Path::new(dir);
    if !path.exists() {
        fs::create_dir_all(path).unwrap();
    } else if path.exists() && !path.is_dir() {
        panic!(format!("Attempted to create directory {:?} but found a regular file", path));
    }
}

pub fn walk_dir(dir: &str) -> Result<Vec<PathBuf>, String> {
    match fs::walk_dir(dir) {
        Ok(walker) => { Ok(walker.map(|x| x.unwrap().path()).collect()) },
        Err(_) => { Err(format!("Failed to traverse directory '{}'", dir)) }
    }
}

/// Walk the given directory, and return all paths ending with the given filename
pub fn find_matching_files(filename: &str, dir: &str) -> Result<Vec<PathBuf>, String> {
    let path = Path::new(filename);
    let dirs = try!(walk_dir(dir));
    Ok(dirs.iter()
        .filter(|p| p.ends_with(path))
        .map(|p| p.clone())
        .collect())
}

/// ensure_extension("aaa", "yml") -> "abc.yml"
/// ensure_extension("aaa.", ".yml") -> "abc.yml"
/// ensure_extension("aaa.yml", "yml") -> "abc.yml"
/// ensure_extension("aaa.poo", "yml") -> "abc.poo.yml"
pub fn ensure_extension(filename: &str, extension: &str) -> String {
    let extension = extension.trim_left_matches('.');
    let filename = filename.trim_right_matches('.');
    if filename.ends_with(extension) {
        filename.to_string()
    } else {
        filename.to_string() + "." + extension
    }
}
