use std::path::Path;
use std::fs::PathExt;
use super::file;
use yaml_rust;
use yaml_rust::Yaml;
use yaml_rust::YamlEmitter;

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
    file::read_file(ACTIVE_ENV_FILE)
}

/// Writes to the active environment file the name of the supplied environment, if it exists.
pub fn set_active_environment(name: &str) {
    let env_filename = &format!("{}/{}", ENV_DIR, file::ensure_extension(name, "yml"));
    if !Path::new(env_filename).exists() {
        panic!("Tried to activate non-existent environment {:?}", name);
    }
    // write out new name to the active file
    file::write_file(ACTIVE_ENV_FILE, name);
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
    Ok(file::load_yaml_file(filename))
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
    let paths = file::find_matching_files(&filename, ".spag/environments");
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
    println!("{}", file::read_file(&filename));
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
    Ok(())
}

/// Loads the environment, unsets a list of keys, and writes out the environment
/// This supports nested key paths:
///     unset_in_environment("default", ["a.b.c", "wumbo"])
///         -> default["a"]["b"]["c"] = None
///         -> default["wumbo"] = None
pub fn unset_in_environment(name: &str, keys: &Vec<String>) -> Result<(), String> {
    file::ensure_dir_exists(ENV_DIR);
    let filename = try!(get_environment_filename(name));
    let mut y = file::load_yaml_file(&filename);

    for key in keys.iter() {
        let parts: Vec<&str> = key.split('.').collect();
        unset_nested_value(&mut y, parts.as_slice());
    }

    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&y).unwrap();
    }
    file::write_file(&filename, &out_str);
    Ok(())
}

/// Empties the environment. Unsets all values.
pub fn unset_all_environment(name: &str) -> Result<(), String> {
    file::ensure_dir_exists(ENV_DIR);
    let filename = try!(get_environment_filename(name));

    file::write_file(&filename, "---\n{}");
    Ok(())
}

/// If keys is ["a", "b", "c"], then set y["a"]["b"]["c"] = <val>. This will create all of the
/// intermediate maps if they don't exist.
pub fn set_nested_value(y: &mut Yaml, keys: &[&str], val: &str) {
    if keys.is_empty() {
        panic!("BUG: No keys given to set in the environment.");
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

/// If keys is ["a", "b", "c"], then unset y["a"]["b"]["c"]
pub fn unset_nested_value(y: &mut Yaml, keys: &[&str]) {
    if keys.is_empty() {
        panic!("BUG: No keys given to unset in the environment.");
    }
    let key = Yaml::String(keys[0].to_string());
    if let Yaml::Hash(ref mut h) = *y {
        if keys.len() == 1 {
            h.remove(&key);
        } else {
            // traverse nested dictionaries to find the key
            unset_nested_value(h.get_mut(&key).unwrap(), &keys[1..]);
        }
    } else {
        panic!(format!("Failed to unset key {:?} in {:?}", key, y));
    }
}

pub fn get_nested_value<'a>(y: &'a Yaml, keys: &[&str]) -> Option<&'a Yaml> {
    if keys.is_empty() { return None; }
    let key = Yaml::String(keys[0].to_string());
    if let Yaml::Hash(ref h) = *y {
        match h.get(&key) {
            Some(val) => {
                if keys.len() == 1 {
                    Some(val)
                } else {
                    get_nested_value(val, &keys[1..])
                }
            },
            None => { None },
        }
    } else {
        None
    }
}
