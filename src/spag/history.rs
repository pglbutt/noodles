use std::io::prelude::*;
use std::path::Path;
use std::str::from_utf8;

use curl::http;
use yaml_rust::YamlEmitter;
use yaml_rust::YamlLoader;
use yaml_rust::Yaml;

use super::file;
use super::env;
use super::request::SpagRequest;

const HISTORY_FILE: &'static str = ".spag/history.yml";
const HISTORY_LIMIT: usize = 100;

pub fn append(req: &SpagRequest, resp: http::Response) -> Result<(), String> {
    if !Path::new(HISTORY_FILE).exists() {
        file::write_file(HISTORY_FILE, "[]");
    }

    let mut y = &mut try!(file::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {

        // Trim the history, -1 so that our request fits under the limit
        if arr.len() > HISTORY_LIMIT - 1 {
            while arr.len() > HISTORY_LIMIT - 1 {
                arr.remove(0);
            }
        }

        let mut outer_y = YamlLoader::load_from_str("{}").unwrap();
        let inner_y = &mut outer_y[0];

        // Add the request data
        env::set_nested_value(inner_y, &["request", "verb"], req.get_method_string());
        env::set_nested_value(inner_y, &["request", "uri"], req.uri.as_str());
        env::set_nested_value(inner_y, &["request", "endpoint"], req.endpoint.as_str());
        env::set_nested_value(inner_y, &["request", "body"], req.body.as_str());

        for (key, value) in &req.headers {
            env::set_nested_value(inner_y, &["request", "headers", key], value.as_str());
        }

        // Add the response data
        env::set_nested_value(inner_y, &["response", "body"], from_utf8(resp.get_body()).unwrap());
        env::set_nested_value(inner_y, &["response", "status"], resp.get_code().to_string().as_str());
        for (key, value) in resp.get_headers() {
            env::set_nested_value(inner_y, &["response", "headers", key], value[0].as_str());
        }

        arr.insert(0, inner_y.clone());
    }

    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&y).unwrap();
    }
    file::write_file(HISTORY_FILE, out_str.as_str());

    Ok(())
}

pub fn list() -> Result<Vec<String>, String> {
    if !Path::new(HISTORY_FILE).exists() {
        file::write_file(HISTORY_FILE, "[]");
    }

    let mut result = Vec::new();

    let mut y = &mut try!(file::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {
        for y in arr.iter() {
            let verb = try!(get_string_from_yaml(&y, &["request", "verb"]));
            let endpoint = try!(get_string_from_yaml(&y, &["request", "endpoint"]));
            let uri = try!(get_string_from_yaml(&y, &["request", "uri"]));
            let s = format!("{} {}{}", verb, endpoint, uri);
            result.push(s);
        }
    }

    Ok(result)
}

pub fn get(raw_index: &String) -> Result<String, String> {
    if !Path::new(HISTORY_FILE).exists() {
        file::write_file(HISTORY_FILE, "[]");
    }

    let index = raw_index.parse().unwrap();

    let mut y = &mut try!(file::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {
        let target = match arr.get(index) {
            Some(yaml) => yaml,
            None => return Err(format!("No request at #{}", index)),
        };

        // Request data
        let mut output = "-------------------- Request ---------------------\n".to_string();
        let verb = try!(get_string_from_yaml(&target, &["request", "verb"]));
        let endpoint = try!(get_string_from_yaml(&target, &["request", "endpoint"]));
        let uri = try!(get_string_from_yaml(&target, &["request", "uri"]));
        let body = try!(get_string_from_yaml(&target, &["request", "body"]));

        output.push_str(format!("{} {}{}\n", verb, endpoint, uri).as_str());
        match env::get_nested_value(&target, &["request", "headers"]) {
            Some(&Yaml::Hash(ref headers)) => {
                for (key, value) in headers.iter() {
                    output.push_str(format!("{}: {}\n", key.as_str().unwrap(),
                                            value.as_str().unwrap()).as_str());
                }
            },
            None => {},
            _ => { return Err(format!("Invalid headers in request history #{}.", index))},
        };

        output.push_str(format!("Body:\n{}\n", body).as_str());
        // Response Data
        output.push_str("-------------------- Response ---------------------\n");

        let body = try!(get_string_from_yaml(&target, &["response", "body"]));
        let status = try!(get_string_from_yaml(&target, &["response", "status"]));

        output.push_str(format!("Status code {}\n", status).as_str());
        match env::get_nested_value(&target, &["response", "headers"]) {
            Some(&Yaml::Hash(ref headers)) => {
                for (key, value) in headers.iter() {
                    output.push_str(format!("{}: {}\n", key.as_str().unwrap(),
                                            value.as_str().unwrap()).as_str());
                }
            },
            None => {},
            _ => { return Err(format!("Invalid headers in request history #{}.", index))},
        };
        output.push_str(format!("Body:\n{}\n", body).as_str());

        Ok(output.to_string())
    } else {
        Err(format!("Failed to load history file {}", HISTORY_FILE))
    }
}

fn get_string_from_yaml(y: &Yaml, keys: &[&str]) -> Result<String, String> {
    match env::get_nested_value(y, keys) {
        Some(&Yaml::String(ref m)) => { Ok(m.to_string()) },
        Some(ref s) => {
            Err(format!("Invalid value '{:?}' for key {:?} in request file", s, keys))
        },
        _ => {
            Err(format!("Missing key {:?} in request file", keys))
        },
    }
}
