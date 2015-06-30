use std::str::from_utf8;
use std::path::Path;

use rustc_serialize::json::Json;
use curl::http;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use super::request::SpagRequest;
use super::file;
use super::yaml_util;

const REMEMBERS_DIR: &'static str = ".spag/remembers";

pub fn remember(req: &SpagRequest, resp: &http::Response) -> Result<(), String> {
    file::ensure_dir_exists(REMEMBERS_DIR);
    let y = serialize(req, resp);
    let output_file = Path::new(REMEMBERS_DIR).join("last.yml");
    yaml_util::dump_yaml_file(output_file.to_str().unwrap(), &y)
}

pub fn serialize(req: &SpagRequest, resp: &http::Response) -> Yaml {
    let mut inner_y = YamlLoader::load_from_str("{}").unwrap().remove(0);

    // Add the request data
    yaml_util::set_nested_value(&mut inner_y, &["request", "method"], req.get_method_string());
    yaml_util::set_nested_value(&mut inner_y, &["request", "uri"], req.uri.as_str());
    yaml_util::set_nested_value(&mut inner_y, &["request", "endpoint"], req.endpoint.as_str());
    yaml_util::set_nested_value(&mut inner_y, &["request", "body"], req.body.as_str());

    for (key, value) in &req.headers {
        yaml_util::set_nested_value(&mut inner_y, &["request", "headers", key], value.as_str());
    }

    // Add the response data
    yaml_util::set_nested_value(&mut inner_y, &["response", "body"], from_utf8(resp.get_body()).unwrap());
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
            match body.find_path(json_key_path) {
                Some(&Json::String(ref s))  => Ok(s.to_string()),
                Some(&Json::I64(val))       => Ok(format!("{}", val)),
                Some(&Json::U64(val))       => Ok(format!("{}", val)),
                Some(&Json::F64(val))       => Ok(format!("{}", val)),
                Some(&Json::Boolean(val))   => Ok(format!("{}", val)),
                Some(&Json::Null)           => Ok("null".to_string()),
                Some(&Json::Array(_)) | Some(&Json::Object(_)) => {
                    Err(format!("Refusing to interpolate json array or object in template"))
                },
                None => Err(format!("Failed to find path {:?} in json request/response body", json_key_path)),
            }
        } else {
            Err(format!("Failed to load body as json for {:?} in remembered request {}", key_path, remembered_name))
        }
    } else {
        yaml_util::get_value_as_string(&y, key_path)
    }
}
