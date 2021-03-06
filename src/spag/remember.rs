use std::str;
use std::path::Path;

use rustc_serialize::json::Json;
use curl::http;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use super::request::SpagRequest;
use super::file;
use super::yaml_util;

const REMEMBERS_DIR: &'static str = ".spag/remembers";

pub fn remember(req: &SpagRequest, resp: &http::Response, remember_as: &str) -> Result<(), String> {
    file::ensure_dir_exists(REMEMBERS_DIR);
    let y = serialize(req, resp);
    let name = file::ensure_extension(remember_as, ".yml");
    let output_file = Path::new(REMEMBERS_DIR).join(name);
    yaml_util::dump_yaml_file(output_file.to_str().unwrap(), &y)
}

pub fn serialize(req: &SpagRequest, resp: &http::Response) -> Yaml {
    let mut inner_y = YamlLoader::load_from_str("{}").unwrap().remove(0);

    // Add the request data
    yaml_util::set_nested_value(&mut inner_y, &["request", "method"], req.get_method_string());
    yaml_util::set_nested_value(&mut inner_y, &["request", "uri"], req.uri.as_str());
    yaml_util::set_nested_value(&mut inner_y, &["request", "endpoint"], req.endpoint.as_str());

    let pretty_req_body = yaml_util::pretty_json(req.body.as_str());
    yaml_util::set_nested_value(&mut inner_y, &["request", "body"], pretty_req_body.as_str());

    for (key, value) in &req.headers {
        yaml_util::set_nested_value(&mut inner_y, &["request", "headers", key], value.as_str());
    }

    // Add the response data
    let pretty_resp_body = yaml_util::pretty_json(str::from_utf8(resp.get_body()).unwrap());
    yaml_util::set_nested_value(&mut inner_y, &["response", "body"], pretty_resp_body.as_str());

    yaml_util::set_nested_value(&mut inner_y, &["response", "status"], resp.get_code().to_string().as_str());
    for (key, value) in resp.get_headers() {
        yaml_util::set_nested_value(&mut inner_y, &["response", "headers", key], value[0].as_str());
    }

    inner_y
}

pub fn load_remembered_request(name: &str) -> Result<Yaml, String> {
    let matches = try!(file::find_matching_files(&file::ensure_extension(name, "yml"), REMEMBERS_DIR));
    if matches.len() == 0 {
        Err(format!("Failed to find remembered request '{}' in {}", name, REMEMBERS_DIR))
    } else if matches.len() == 1 {
        yaml_util::load_yaml_file(matches[0].to_str().unwrap())
    } else {
        Err(format!("Found ambiguous options for remembered request '{}'. Pick on of {:?}", name, matches))
    }
}

/// Load the remembered request and grab a value from it
pub fn find_remembered_key(remembered_name: &str, key_path: &[&str]) -> Result<String, String> {
    let y = try!(load_remembered_request(remembered_name));
    // println!("{:?}", y);

    // if we're grabbing a value out of the request body, load it as json
    if key_path.len() > 2 && (key_path.starts_with(&["request", "body"]) || key_path.starts_with(&["response", "body"])) {
        let yaml_key_path = &key_path[0..2];
        let json_key_path = &key_path[2..];
        let body_string = try!(yaml_util::get_value_as_string(&y, yaml_key_path));
        // println!("\nbody_string {:?}", body_string);
        if let Ok(body) = Json::from_str(&body_string) {
            // println!("body: {:?}", body);
            // TODO: find_path only works on Json::Objects. we need to handle indexing into Json::Arrays.
            match *try!(json_find_path(&body, json_key_path)) {
                Json::String(ref s)  => Ok(s.to_string()),
                Json::I64(val)       => Ok(format!("{}", val)),
                Json::U64(val)       => Ok(format!("{}", val)),
                Json::F64(val)       => Ok(format!("{}", val)),
                Json::Boolean(val)   => Ok(format!("{}", val)),
                Json::Null           => Ok("null".to_string()),
                Json::Array(_) | Json::Object(_) => {
                    Err(format!("Refusing to interpolate json array or object in template"))
                },
            }
        } else {
            Err(format!("Failed to load body as json for {:?} in remembered request {}", key_path, remembered_name))
        }
    } else {
        yaml_util::get_value_as_string(&y, key_path)
    }
}

/// Grab a value out of some json. This handles array indexing properly.
pub fn json_find_path<'a>(data: &'a Json, key_path: &[&str]) -> Result<&'a Json, String> {
    if key_path.is_empty() {
       return Err("Empty json key".to_string());
    }

    let mut target = data;
    for key in key_path.iter() {
        if target.is_array() {
            match key.parse::<usize>() {
                Ok(x) => {
                    if x < target.as_array().unwrap().len() {
                        target = &target[x];
                    } else {
                        return Err(format!("Index {} out of bounds for key path {:?}", key, key_path));
                    }
                },
                Err(_) => { return Err(format!("Invalid array index '{}' for key path {:?}", key, key_path)); },
            };
        } else {
            match target.find(*key) {
                Some(t) => { target = t; },
                None => { return Err(format!("Invalid key '{}'", key)); },
            }
        }
    }
    Ok(target)
}
