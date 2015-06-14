use std::path::Path;
use std::fs::PathExt;
use super::file;
use yaml_rust;
use yaml_rust::Yaml;
use yaml_rust::YamlEmitter;

const ENV_DIR: &'static str          = ".spag/environments";
const ACTIVE_ENV_FILE: &'static str  = ".spag/environments/active";
const DEFAULT_ENV_NAME: &'static str = "default";

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
    file::read_file(ACTIVE_ENV_FILE)
}

pub fn load_environment(name: &str) -> Yaml {
    let filename = &get_environment_filename(name);
    if !Path::new(filename).exists() {
        file::write_file(filename, "{}");
    }
    file::load_yaml_file(filename)
}

fn get_environment_filename(name: &str) -> String {
    let name =
        if name.is_empty() {
            get_active_environment_name()
        } else {
            name.to_string()
        };
    let filename = file::ensure_extension(&name, ".yml");
    let paths = file::find_matching_files(&filename, ".spag/environments");
    if paths.is_empty() {
        panic!("Environment not found");
    } else if paths.len() >= 2 {
        panic!("Ambiguous environment name. Pick one of {:?}", paths);
    } else {
        paths[0].to_str().unwrap().to_string()
    }
}

/// Print out the given environment. If name is empty, use the active environment.
/// The name will be fixed to end with '.yml'
pub fn show_environment(name: &str) {
    file::ensure_dir_exists(ENV_DIR);
    let filename = get_environment_filename(name);
    println!("{}", file::read_file(&filename));
}

/// Loads the environment, sets a list of key-value pairs, and writes out the environment
/// This supports nested key paths:
///     set_in_environment("default", ["a.b.c", "mini], ["efg", "wumbo"])
///         -> default["a"]["b"]["c"] = "efg"
///         -> default["mini"] = "wumbo"
pub fn set_in_environment(name: &str, keys: &Vec<String>, vals: &Vec<String>) {
    file::ensure_dir_exists(ENV_DIR);
    let filename = get_environment_filename(name);
    let mut y = file::load_yaml_file(&filename);

    for (k, v) in keys.iter().zip(vals.iter()) {
        let parts: Vec<&str> = k.split('.').collect();
        set_nested_value(&mut y, parts.as_slice(), &v);
    }

    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&y).unwrap();
    }
    file::write_file(&filename, &out_str);
}


/// If keys is ["a", "b", "c"], then set y["a"]["b"]["c"] = <val>. This will create all of the
/// intermediate maps if they don't exist.
pub fn set_nested_value(y: &mut Yaml, keys: &[&str], val: &str) {
    if keys.is_empty() {
        panic!("BUG: No keys given to set in the Yaml");
    }
    let key = Yaml::String(keys[0].to_string());
    if let Yaml::Hash(ref mut h) = *y {
        if keys.len() == 1 {
            h.insert(key, Yaml::String(val.to_string()));
        } else {
            // create nested dictionaries if they don't exist
            if let None = h.get_mut(&key) {
                h.insert(key.clone(), Yaml::Hash(yaml_rust::yaml::Hash::new()));
            }
            set_nested_value(h.get_mut(&key).unwrap(), &keys[1..], val);
        }
    } else {
        panic!(format!("Failed to set key {:?} in {:?}", key, y));
    }
}
