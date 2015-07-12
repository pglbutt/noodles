use std;
use std::io::Write;
use std::path::Path;
use std::fs::PathExt;

use yaml_rust::Yaml;

use super::file;
use super::yaml_util;

const ENV_DIR: &'static str          = ".spag/environments";
const ACTIVE_ENV_FILE: &'static str  = ".spag/environments/active";
const DEFAULT_ENV_NAME: &'static str = "default";

/// Creates the active environment file, and the default environment file if they don't exist.
/// Returns the name of the active environment, read from the active environment file.
pub fn get_active_environment_name() -> String {
    // create file specifiying the active env if it doesn't exist
    if !Path::new(ACTIVE_ENV_FILE).exists() {
        file::write_file(ACTIVE_ENV_FILE, DEFAULT_ENV_NAME);
    }

    // create the default environment if it doesn't exist
    let default_file = &format!("{}/{}", ENV_DIR, file::ensure_extension(DEFAULT_ENV_NAME, "yml"));
    if !Path::new(default_file).exists() {
        file::write_file(default_file, "{}");
    }
    try_error!(file::read_file(ACTIVE_ENV_FILE))
}

/// Writes to the active environment file the name of the supplied environment, if it exists.
pub fn set_active_environment(name: &str) -> Result<(), String>{
    let env_filename = &format!("{}/{}", ENV_DIR, file::ensure_extension(name, "yml"));
    if !Path::new(env_filename).exists() {
        return Err(format!("Tried to activate non-existent environment {:?}", name))
    }
    // write out new name to the active file
    file::write_file(ACTIVE_ENV_FILE, name);
    Ok(())
}

/// Sets the active environment to the 'default' environment.
pub fn deactivate_environment() {
    let activename = get_active_environment_name();

    if activename != DEFAULT_ENV_NAME {
        set_active_environment(DEFAULT_ENV_NAME);
    }
}

/// Returns a YAML object of the environment file requested.
/// If the environment doesn't exist, it creates it.
pub fn load_environment(name: &str) -> Result<Yaml, String> {
    let filename = &try!(get_environment_filename(name));
    if !Path::new(filename).exists() {
        file::write_file(filename, "{}");
    }
    yaml_util::load_yaml_file(filename)
}

/// Returns the filename for the request environment.
/// If an empty string is passed, the active environment filename is returned.
/// Handles ambiguous environment names.
fn get_environment_filename(name: &str) -> Result<String, String> {
    let name =
        if name.is_empty() {
            get_active_environment_name()
        } else {
            name.to_string()
        };
    let filename = file::ensure_extension(&name, ".yml");
    let paths = try!(file::find_matching_files(&filename, ".spag/environments"));
    if paths.is_empty() {
        Err(format!("Environment not found"))
    } else if paths.len() >= 2 {
        Err(format!("Ambiguous environment name. Pick one of {:?}", paths))
    } else {
        Ok(paths[0].to_str().unwrap().to_string())
    }
}

/// Print out the given environment. If name is empty, use the active environment.
/// The name will be fixed to end with '.yml'
pub fn show_environment(name: &str) -> Result<(), String> {
    file::ensure_dir_exists(ENV_DIR);
    let filename = try!(get_environment_filename(name));
    println!("{}", try!(file::read_file(&filename)));
    Ok(())
}

/// Loads the environment, sets a list of key-value pairs, and writes out the environment
/// This supports nested key paths:
///     set_in_environment("default", ["a.b.c", "mini], ["efg", "wumbo"])
///         -> default["a"]["b"]["c"] = "efg"
///         -> default["mini"] = "wumbo"
pub fn set_in_environment(name: &str, keys: &Vec<String>, vals: &Vec<String>
                          ) -> Result<(), String> {
    file::ensure_dir_exists(ENV_DIR);
    let filename = try!(get_environment_filename(name));
    let mut y = try!(yaml_util::load_yaml_file(&filename));

    for (k, v) in keys.iter().zip(vals.iter()) {
        let parts: Vec<&str> = k.split('.').collect();
        yaml_util::set_nested_value(&mut y, parts.as_slice(), &v);
    }

    Ok(try!(yaml_util::dump_yaml_file(&filename, &y)))
}

/// Loads the environment, unsets a list of keys, and writes out the environment
/// This supports nested key paths:
///     unset_in_environment("default", ["a.b.c", "wumbo"])
///         -> default["a"]["b"]["c"] = None
///         -> default["wumbo"] = None
pub fn unset_in_environment(name: &str, keys: &Vec<String>) -> Result<(), String> {
    file::ensure_dir_exists(ENV_DIR);
    let filename = try!(get_environment_filename(name));
    let mut y = try!(yaml_util::load_yaml_file(&filename));

    for key in keys.iter() {
        let parts: Vec<&str> = key.split('.').collect();
        yaml_util::unset_nested_value(&mut y, parts.as_slice());
    }

    Ok(try!(yaml_util::dump_yaml_file(&filename, &y)))
}

/// Empties the environment. Unsets all values.
pub fn unset_all_environment(name: &str) -> Result<(), String> {
    file::ensure_dir_exists(ENV_DIR);
    let filename = try!(get_environment_filename(name));

    file::write_file(&filename, "---\n{}");
    Ok(())
}
