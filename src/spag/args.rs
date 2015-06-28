extern crate curl;
extern crate docopt;

use std;
use std::io::Write;
use std::collections::hash_map::HashMap;

use curl::http::handle::Method;
use docopt::Docopt;
use yaml_rust::Yaml;
use yaml_rust::yaml::Hash;

use super::request;
use super::env;
use super::template;

// docopt! gives us a non-public struct. this renames that struct and makes it public.
pub type Args = ArgsPrivate;

docopt!(ArgsPrivate derive Debug, "
Usage:
    spag --help
    spag env set (<key> <val>)...
    spag env unset [(<key>)...] [-E]
    spag env show [<environment>]
    spag env activate <environment>
    spag env deactivate
    spag (get|post|put|patch|delete) <resource> [(-H <header>)...] [-e <endpoint>] [-d <data>] [(--with <key> <val>)...]
    spag request list [--dir <dir>]
    spag request show <file>
    spag request <file> [(-H <header>)...] [-e <endpoint>] [-d <data>] [(--with <key> <val>)...] [--dir <dir>]
    spag history
    spag history show <index>

Options:
    -h, --help                  Show this message
    -H, --header <header>       Supply a header
    -e, --endpoint <endpoint>   Supply the endpoint
    -d, --data <data>           Supply the request body
    -E, --everything            Unset an entire environment
    --dir <dir>                 The directory containing request files

Arguments:
    <endpoint>      The base url of the service, like 'http://localhost:5000'
    <resource>      The path of an api resource, like '/v2/things'
    <header>        An http header, like 'Content-type: application/json'
    <environment>   The name of an environment, like 'default'
    <index>         An index, starting at zero

Commands:
    env set         Set a key-value pair in the active environment
    env unset       Unset one or more keys in the active environment
    env show        Print out the specified environment
    env activate    Activate an environment by name
    env deactivate  Deactivate the environment and return to the default environment
    get             An HTTP GET request
    post            An HTTP POST request
    put             An HTTP PUT request
    patch           An HTTP PATCH request
    delete          An HTTP DELETE request
    request         Make a request using a predefined file
    request show    Show the specified request file
    request list    List available request files
    history         Print a list of previously made requests
    history show    Print out a previous request by its index
");

pub fn parse_args() -> Args {
    Args::docopt().decode().unwrap_or_else(|e| e.exit())
}

pub fn get_method_from_args(args: &Args) -> Method {
    if args.cmd_get { Method::Get }
    else if args.cmd_post { Method::Post }
    else if args.cmd_put { Method::Put }
    else if args.cmd_patch { Method::Patch }
    else if args.cmd_delete { Method::Delete }
    else { panic!("BUG: method not recognized"); }
}

pub fn get_endpoint(args: &Args) -> Result<String, String> {
    // passing -e ENDPOINT overrides everything else
    if !args.flag_endpoint.is_empty() {
        Ok(args.flag_endpoint.to_string())
    } else {
        let env = try!(env::load_environment(""));
        if let Some(e) = env["endpoint"].as_str() {
            Ok(e.to_string())
        } else {
            Err("Endpoint not set".to_string())
        }
    }
}

pub fn get_dir(args: &Args) -> Result<String, String> {
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

pub fn get_data(args: &Args) -> Result<String, String> {
    let use_shortcuts = true;
    let withs: HashMap<String, String> = get_withs(args);
    let withs: HashMap<&str, &str> = withs.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    Ok(try!(template::untemplate(&args.flag_data, &withs, use_shortcuts)))
}

fn get_headers_from_request(request_yaml: &Yaml) -> Result<HashMap<String, String>, String> {
    let default_hash = &Yaml::Hash(Hash::new());
    let mut result: HashMap<String, String> = HashMap::new();
    let request_file_headers = env::get_nested_value(&request_yaml, &["headers"]).unwrap_or(default_hash);
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
    let env_headers = env::get_nested_value(&env, &["headers"]).unwrap_or(default_hash);
    if let &Yaml::Hash(ref h) = env_headers {
        for (k, v) in h.iter() {
            if let (&Yaml::String(ref key), &Yaml::String(ref value)) = (k, v) {
                result.insert(key.to_string(), value.to_string());
            }
        }
    }
    Ok(result)
}

fn get_headers_from_args(args: &Args) -> Result<HashMap<String, String>, String> {
    let use_shortcuts = true;
    let mut result: HashMap<String, String> = HashMap::new();
    let arg_headers: Vec<(&str, &str)> =
        try_error!(args.flag_header.iter().map(|s| request::split_header(s)).collect());
    for &(k, v) in arg_headers.iter() {
        let v = try_error!(template::untemplate(&v, &HashMap::new(), use_shortcuts));
        result.insert(k.to_string(), v.to_string());
    }
    Ok(result)
}

/// Build a single list of headers from the environment, the request yaml, and arguments.
pub fn resolve_headers(args: &Args, request_yaml: &Yaml) -> Result<Vec<String>, String> {
    let request_headers = try!(get_headers_from_request(request_yaml));
    let env_headers = try!(get_headers_from_environment());
    let arg_headers = try!(get_headers_from_args(args));
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

pub fn resolve_headers_no_request_file(args: &Args) -> Result<Vec<String>, String> {
    resolve_headers(args, &Yaml::Hash(Hash::new()))
}

pub fn get_string_from_yaml(y: &Yaml, keys: &[&str]) -> String {
    match env::get_nested_value(y, keys) {
        Some(&Yaml::String(ref m)) => { m.to_string() },
        Some(ref s) => {
            error!("Invalid value '{:?}' for key {:?} in request file", s, keys);
        },
        _ => {
            error!("Missing key {:?} in request file", keys);
        },
    }
}

pub fn get_withs(args: &Args) -> HashMap<String, String> {
//    println!("key={:?} val={:?}", args.arg_key, args.arg_val);
    let use_shortcuts = true;
    let mut withs = HashMap::new();
    for (k, v) in args.arg_key.iter().zip(args.arg_val.iter()) {
        let v = try_error!(template::untemplate(&v, &HashMap::new(), use_shortcuts));
        withs.insert(k.to_string(), v.to_string());
    }
//    println!("{:?}", withs);
    withs
}
