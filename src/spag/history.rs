use std::io::prelude::*;
use std::path::Path;
use std::str::from_utf8;

use curl::http;
use yaml_rust::YamlLoader;
use yaml_rust::Yaml;

use super::file;
use super::yaml_util;
use super::request::SpagRequest;

const HISTORY_FILE: &'static str = ".spag/history.yml";
const HISTORY_LIMIT: usize = 100;

pub fn append(req: &SpagRequest, resp: http::Response) -> Result<(), String> {
    if !Path::new(HISTORY_FILE).exists() {
        file::write_file(HISTORY_FILE, "[]");
    }

    let mut y = &mut try!(yaml_util::load_yaml_file(&HISTORY_FILE));

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
        yaml_util::set_nested_value(inner_y, &["request", "verb"], req.get_method_string());
        yaml_util::set_nested_value(inner_y, &["request", "uri"], req.uri.as_str());
        yaml_util::set_nested_value(inner_y, &["request", "endpoint"], req.endpoint.as_str());
        yaml_util::set_nested_value(inner_y, &["request", "body"], req.body.as_str());

        for (key, value) in &req.headers {
            yaml_util::set_nested_value(inner_y, &["request", "headers", key], value.as_str());
        }

        // Add the response data
        yaml_util::set_nested_value(inner_y, &["response", "body"], from_utf8(resp.get_body()).unwrap());
        yaml_util::set_nested_value(inner_y, &["response", "status"], resp.get_code().to_string().as_str());
        for (key, value) in resp.get_headers() {
            yaml_util::set_nested_value(inner_y, &["response", "headers", key], value[0].as_str());
        }

        arr.insert(0, inner_y.clone());
    }

    Ok(try!(yaml_util::dump_yaml_file(HISTORY_FILE, &y)))
}

pub fn list() -> Result<Vec<String>, String> {
    if !Path::new(HISTORY_FILE).exists() {
        file::write_file(HISTORY_FILE, "[]");
    }

    let mut result = Vec::new();

    let mut y = &mut try!(yaml_util::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {
        for y in arr.iter() {
            let verb = try!(yaml_util::get_value_as_string(&y, &["request", "verb"]));
            let endpoint = try!(yaml_util::get_value_as_string(&y, &["request", "endpoint"]));
            let uri = try!(yaml_util::get_value_as_string(&y, &["request", "uri"]));
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

    let mut y = &mut try!(yaml_util::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {
        let target = match arr.get(index) {
            Some(yaml) => yaml,
            None => return Err(format!("No request at #{}", index)),
        };

        // Request data
        let mut output = "-------------------- Request ---------------------\n".to_string();
        let verb = try!(yaml_util::get_value_as_string(&target, &["request", "verb"]));
        let endpoint = try!(yaml_util::get_value_as_string(&target, &["request", "endpoint"]));
        let uri = try!(yaml_util::get_value_as_string(&target, &["request", "uri"]));
        let body = try!(yaml_util::get_value_as_string(&target, &["request", "body"]));

        output.push_str(format!("{} {}{}\n", verb, endpoint, uri).as_str());
        match yaml_util::get_nested_value(&target, &["request", "headers"]) {
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

        let body = try!(yaml_util::get_value_as_string(&target, &["response", "body"]));
        let status = try!(yaml_util::get_value_as_string(&target, &["response", "status"]));

        output.push_str(format!("Status code {}\n", status).as_str());
        match yaml_util::get_nested_value(&target, &["response", "headers"]) {
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
