use std::io::prelude::*;
use std::path::Path;

use curl::http;
use yaml_rust::Yaml;

use super::file;
use super::yaml_util;
use super::request::SpagRequest;
use super::remember;

const HISTORY_DIR: &'static str = ".spag";
const HISTORY_FILE: &'static str = ".spag/history.yml";
const HISTORY_LIMIT: usize = 100;

pub fn ensure_history_exists() {
    if !Path::new(HISTORY_FILE).exists() {
        file::ensure_dir_exists(HISTORY_DIR);
        file::write_file(HISTORY_FILE, "[]");
    }
}

pub fn append(req: &SpagRequest, resp: &http::Response) -> Result<(), String> {
    ensure_history_exists();

    let mut y = &mut try!(yaml_util::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {

        // Trim the history, -1 so that our request fits under the limit
        if arr.len() > HISTORY_LIMIT - 1 {
            while arr.len() > HISTORY_LIMIT - 1 {
                arr.remove(0);
            }
        }

        let new_entry = remember::serialize(req, resp);

        arr.insert(0, new_entry);
    }

    Ok(try!(yaml_util::dump_yaml_file(HISTORY_FILE, &y)))
}

pub fn list() -> Result<Vec<String>, String> {
    ensure_history_exists();

    let mut result = Vec::new();

    let mut y = &mut try!(yaml_util::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {
        for y in arr.iter() {
            let method = try!(yaml_util::get_value_as_string(&y, &["request", "method"]));
            let endpoint = try!(yaml_util::get_value_as_string(&y, &["request", "endpoint"]));
            let uri = try!(yaml_util::get_value_as_string(&y, &["request", "uri"]));
            let s = format!("{} {}{}", method, endpoint, uri);
            result.push(s);
        }
    }

    Ok(result)
}

pub fn get(raw_index: &String) -> Result<String, String> {
    ensure_history_exists();

    let index = raw_index.parse().unwrap();

    let mut y = &mut try!(yaml_util::load_yaml_file(&HISTORY_FILE));

    if let Yaml::Array(ref mut arr) = *y {
        let target = match arr.get(index) {
            Some(yaml) => yaml,
            None => return Err(format!("No request at #{}", index)),
        };

        // Request data
        let mut output = "-------------------- Request ---------------------\n".to_string();
        let method = try!(yaml_util::get_value_as_string(&target, &["request", "method"]));
        let endpoint = try!(yaml_util::get_value_as_string(&target, &["request", "endpoint"]));
        let uri = try!(yaml_util::get_value_as_string(&target, &["request", "uri"]));
        let body = try!(yaml_util::get_value_as_string(&target, &["request", "body"]));

        output.push_str(format!("{} {}{}\n", method, endpoint, uri).as_str());
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
