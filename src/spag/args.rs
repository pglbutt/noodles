extern crate curl;
extern crate docopt;

use std;
use std::io::prelude::*;
use std::collections::hash_map::HashMap;

use curl::http::handle::Method;
use docopt::Docopt;
use yaml_rust::Yaml;
use yaml_rust::yaml::Hash;

use super::request;
use super::env;
use super::template;
use super::yaml_util;

docopt!(pub MainArgs derive Debug, "
Usage:
    spag [options]
    spag <command> [<args>...]

Options:
    -h --help       Show this message

Commands:
    env             Manage spag environments
    request         Send predefined request files
    history         View request history
    <method>        Perform an HTTP request: get, post, patch, delete, etc
");

docopt!(pub EnvArgs derive Debug, "
Usage:
    spag env --help
    spag env ls
    spag env cat [<environment>]
    spag env activate <environment>
    spag env deactivate
    spag env set (<key> <val>)...
    spag env unset [(<key>)...] [-E]

Options:
    -h --help           Show this message
    -E --everything     Unset an entire environment

Arguments:
    <environment>       The name of an environment, like 'default'
    <key>               The key name of a value to set, like 'headers.Content-type'
    <val>               The value to set on the given key
");

docopt!(pub RequestArgs derive Debug, "
Usage:
    spag (request|r) --help
    spag (request|r) ls [--dir <dir>]
    spag (request|r) cat <file>
    spag (request|r) inspect <file>
    spag (request|r) <file> [options] [(-H <header>)...] [(-w|--with <key> <val>)...]

Options:
    -h --help                   Show this message
    -H --header <header>        Supply a header
    -e --endpoint <endpoint>    Supply the endpoint
    -d --data <data>            Supply the request body
    -v --verbose                Print out more of the request and response
    -r --remember-as <name>     Additionally, remember this request under the given name
    --dir <dir>                 The directory containing request files

Arguments:
    <endpoint>      The base url of the service, like 'http://localhost:5000'
    <header>        An http header, like 'Content-type: application/json'
");

docopt!(pub HistoryArgs derive Debug, "
Usage:
    spag history [options]
    spag history show <index>

Options:
    -h --help       Show this message

Arguments:
    <index>         An index, starting at zero
");

docopt!(pub MethodArgs derive Debug, "
Usage:
    spag <method> --help
    spag <method> <path> [options] [(-H <header>)...]

Options:
    -h --help                   Show this message
    -H --header <header>        Supply a header
    -e --endpoint <endpoint>    Supply the endpoint
    -d --data <data>            Supply the request body
    -v --verbose                Print out more of the request and response
    -r --remember-as <name>     Remember this request under the given name

Arguments:
    <method>        The http method: get, post, put, patch, delete
    <endpoint>      The base url of the service, like 'http://localhost:5000'
    <path>          The path of an api resource, like '/v2/things'
    <header>        An http header, like 'Content-type: application/json'
");

// I tried to find a nicer way to parse args *outside of this module*, but MainArgs::docopt() is
// private, which means we never refer to it outside of this module. So we have a bunch of new
// functions we can call into to do things for us.
pub fn parse_main_args(args: &Vec<String>) -> MainArgs { parse_args!(MainArgs, args) }
pub fn parse_env_args(args: &Vec<String>) -> EnvArgs { parse_args!(EnvArgs, args) }
pub fn parse_request_args(args: &Vec<String>) -> RequestArgs { parse_args!(RequestArgs, args) }
pub fn parse_method_args(args: &Vec<String>) -> MethodArgs { parse_args!(MethodArgs, args) }
pub fn parse_history_args(args: &Vec<String>) -> HistoryArgs { parse_args!(HistoryArgs, args) }

pub fn get_method_from_args(args: &MethodArgs) -> Method {
    match args.arg_method.to_lowercase().as_str() {
        "get" => Method::Get,
        "post" => Method::Post,
        "put" => Method::Put,
        "patch" => Method::Patch,
        "delete" => Method::Delete,
        _ => { panic!("BUG: method not recognized"); },
    }
}

pub fn get_endpoint(flag_endpoint: &str) -> Result<String, String> {
    // passing -e ENDPOINT overrides everything else
    if !flag_endpoint.is_empty() {
        Ok(flag_endpoint.to_string())
    } else {
        let env = try!(env::load_environment(""));
        if let Some(e) = env["endpoint"].as_str() {
            Ok(e.to_string())
        } else {
            Err("Endpoint not set".to_string())
        }
    }
}

pub fn get_dir(args: &RequestArgs) -> Result<String, String> {
    // passing in --dir overrides everything else
    if !args.flag_dir.is_empty() {
        Ok(args.flag_dir.to_string())
    } else {
        let env = try!(env::load_environment(""));
        if let Some(e) = env["dir"].as_str() {
            Ok(e.to_string())
        } else {
            Err("Request directory not set".to_string())
        }
    }
}

pub fn get_data(flag_data: &str, withs: &HashMap<&str, &str>) -> Result<String, String> {
    let use_shortcuts = true;
    Ok(try!(template::untemplate(flag_data, &withs, use_shortcuts)))
}

fn get_headers_from_request(request_yaml: &Yaml) -> Result<HashMap<String, String>, String> {
    let default_hash = &Yaml::Hash(Hash::new());
    let mut result: HashMap<String, String> = HashMap::new();
    let request_file_headers = yaml_util::get_nested_value(&request_yaml, &["headers"]).unwrap_or(default_hash);
    if let &Yaml::Hash(ref h) = request_file_headers {
        for (k, v) in h.iter() {
            if let (&Yaml::String(ref key), &Yaml::String(ref value)) = (k, v) {
                result.insert(key.to_string(), value.to_string());
            }
        }
    }
    Ok(result)
}

fn get_headers_from_environment() -> Result<HashMap<String, String>, String> {
    let default_hash = &Yaml::Hash(Hash::new());
    let mut result: HashMap<String, String> = HashMap::new();
    // TODO: case insensitivity
    // be sure not to fail if we fail to load the env.
    let env = env::load_environment("").unwrap_or(Yaml::Hash(Hash::new()));
    let env_headers = yaml_util::get_nested_value(&env, &["headers"]).unwrap_or(default_hash);
    if let &Yaml::Hash(ref h) = env_headers {
        for (k, v) in h.iter() {
            if let (&Yaml::String(ref key), &Yaml::String(ref value)) = (k, v) {
                result.insert(key.to_string(), value.to_string());
            }
        }
    }
    Ok(result)
}

fn get_headers_from_args(flag_header: &Vec<String>) -> Result<HashMap<String, String>, String> {
    let use_shortcuts = true;
    let mut result: HashMap<String, String> = HashMap::new();
    let arg_headers: Vec<(&str, &str)> =
        try_error!(flag_header.iter().map(|s| request::split_header(s)).collect());
    for &(k, v) in arg_headers.iter() {
        let v = try_error!(template::untemplate(&v, &HashMap::new(), use_shortcuts));
        result.insert(k.to_string(), v.to_string());
    }
    Ok(result)
}

/// Build a single list of headers from the environment, the request yaml, and arguments.
pub fn resolve_headers(arg_headers: &Vec<String>, request_yaml: &Yaml) -> Result<Vec<String>, String> {
    let request_headers = try!(get_headers_from_request(request_yaml));
    let env_headers = try!(get_headers_from_environment());
    let arg_headers = try!(get_headers_from_args(arg_headers));
    let mut result: HashMap<String, String> = HashMap::new();
    // start with headers in the environment
    result.extend(env_headers);
    // headers in the request override headers in the environment
    result.extend(request_headers);
    // headers in arguments override everything
    result.extend(arg_headers);

    // format headers as "<key>: <val>"
    let str_headers: Vec<String> = result.iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect();
    Ok(str_headers)
}

pub fn resolve_headers_no_request_file(flag_header: &Vec<String>) -> Result<Vec<String>, String> {
    resolve_headers(flag_header, &Yaml::Hash(Hash::new()))
}

pub fn get_withs(keys: &Vec<String>, vals: &Vec<String>) -> HashMap<String, String> {
    let use_shortcuts = true;
    let mut withs = HashMap::new();
    for (k, v) in keys.iter().zip(vals.iter()) {
        let v = try_error!(template::untemplate(&v, &HashMap::new(), use_shortcuts));
        withs.insert(k.to_string(), v.to_string());
    }
    withs
}
