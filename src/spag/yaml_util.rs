use yaml_rust;
use yaml_rust::Yaml;
use yaml_rust::yaml::Hash;
use yaml_rust::YamlEmitter;
use yaml_rust::YamlLoader;
use rustc_serialize::json;

use super::file;

pub fn load_yaml_string(s: &str) -> Result<Yaml, String> {
    match YamlLoader::load_from_str(s) {
        Ok(yaml_docs) => { Ok(yaml_docs[0].clone()) }
        Err(msg) => { Err(format!("Failed to load yaml {:?}\n{}", msg, s)) }
    }
}

pub fn dump_yaml_string(y: &Yaml) -> Result<String, String> {
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        match emitter.dump(&y) {
            Err(e) => { return Err(format!("Error while writing yaml string -- {:?}", e)); },
            _ => {},
        }
    }
    Ok(out_str)
}

pub fn load_yaml_file(filename: &str) -> Result<Yaml, String> {
    let s = try!(file::read_file(filename));
    match load_yaml_string(&s) {
        Err(msg) => { Err(format!("Failed to load yaml file {}\n{:?}", filename, msg)) },
        x => x,
    }
}

pub fn dump_yaml_file(filename: &str, y: &Yaml) -> Result<(), String> {
    let yaml_string = try!(dump_yaml_string(y));
    file::write_file(&filename, &yaml_string);
    Ok(())
}

pub fn get_value_as_string(y: &Yaml, keys: &[&str]) -> Result<String, String> {
    match get_nested_value(y, keys) {
        Some(&Yaml::String(ref m)) => { Ok(m.to_string()) },
        Some(ref s) => {
            Err(format!("Invalid value '{:?}' for key {:?}", s, keys))
        },
        _ => {
            Err(format!("Missing key {:?}", keys))
        },
    }
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

// If body can be serialized to JSON, return a pretty JSON string, otherwise return original string 
pub fn pretty_json(resp_output: &str) -> String {
    match json::Json::from_str(&resp_output) {
        Ok(val) => return format!("{}", val.pretty()),
        Err(_) => return resp_output.to_string(),
    };
}