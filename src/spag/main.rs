extern crate curl;
extern crate docopt;

use std;
use std::io::Write;
use std::collections::hash_map::HashMap;
use std::path::PathBuf;
use curl::http::handle::Method;
use curl::http;
use docopt::Docopt;
use super::request::SpagRequest;
use super::env;
use super::template;
use super::request;
use super::file;

use yaml_rust::Yaml;
use yaml_rust::yaml::Hash;

/// Formats a string which is printed to stderr, and exits with status code 1
///
/// ```
/// error!("Failed to bring it around town: {}", arg);
/// ```
macro_rules! error {
    ($($arg:tt)*) => ({
        match writeln!(&mut std::io::stderr(), $($arg)*) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
        std::process::exit(1);
    })
}

/// Unwraps a Result like try!(), but calls error!("{}", msg) if the result is Err(msg)
///
/// ```
/// let value = try_error!(result);
/// ```
macro_rules! try_error {
    ($expr:expr) => ({
        match $expr {
            Ok(val) => val,
            Err(err) => error!("{}", err),
        }
    })
}

docopt!(Args derive Debug, "
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

pub fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    // println!("{:?}", args);

    if args.cmd_request {
        spag_request(&args);
    } else if args.cmd_history {
        spag_history(&args);
    } else if args.cmd_env {
        spag_env(&args);
    } else if args.cmd_get || args.cmd_post || args.cmd_put || args.cmd_patch || args.cmd_delete {
        spag_method(&args);
    }
}

fn spag_env(args: &Args) {
    if args.cmd_show {
        spag_env_show(&args);
    } else if args.cmd_set {
        spag_env_set(&args);
    } else if args.cmd_unset {
        spag_env_unset(&args);
    } else if args.cmd_activate {
        spag_env_activate(&args);
    } else if args.cmd_deactivate {
        spag_env_deactivate(&args);
    } else {
        panic!("BUG: Invalid command");
    }
}

fn spag_env_set(args: &Args) {
    env::set_in_environment(&args.arg_environment, &args.arg_key, &args.arg_val);
    env::show_environment(&args.arg_environment);
}

fn spag_env_unset(args: &Args) {
    if args.flag_everything == false {
        env::unset_in_environment(&args.arg_environment, &args.arg_key);
        env::show_environment(&args.arg_environment);
    } else {
        env::unset_all_environment(&args.arg_environment);
        env::show_environment(&args.arg_environment);
    }
}

fn spag_env_show(args: &Args) {
    env::show_environment(&args.arg_environment);
}

fn spag_env_activate(args: &Args) {
    env::set_active_environment(&args.arg_environment);
    env::show_environment(&args.arg_environment);
}

fn spag_env_deactivate(args: &Args) {
    env::deactivate_environment();
}

fn spag_history(args: &Args) {
    println!("called spag history");
}


fn spag_request(args: &Args) {

    let endpoint = try_error!(get_endpoint(args));
    let dir = try_error!(get_dir(args));

    if args.cmd_list {
        spag_request_list(args);
        return;
    }
    // TODO: handle cmd_show

    match request::load_request_file(&args.arg_file, &dir) {
        Ok(y) => {
            // TODO: refactor this
            let method = get_string_from_yaml(&y, &["method"]);
            let uri = get_string_from_yaml(&y, &["uri"]);

            let default_string = &Yaml::String(String::new());
            let body = env::get_nested_value(&y, &["body"]).unwrap_or(default_string);
            let default_hash = &Yaml::Hash(Hash::new());
            let headers = env::get_nested_value(&y, &["headers"]).unwrap_or(default_hash);

            if let (&Yaml::String(ref b), &Yaml::Hash(ref h)) = (body, headers) {
                let str_headers: Vec<String> = h.iter()
                    .map(|(k, v)| format!("{}: {}", k.as_str().unwrap(), v.as_str().unwrap()))
                    .collect();
                //println!("{:?}", str_headers);

                let mut req = SpagRequest::new(request::method_from_str(&method), endpoint, uri);
                req.add_headers(str_headers.iter());
                req.set_body(b.to_string());
                do_request(&req);
            } else {
                println!("BUG: bad type on body or header");
                return;
            }
        },
        Err(msg) => { println!("{}", msg); }
    }
    //println!("key={:?} val={:?}", args.arg_key, args.arg_val);
    //let mut withs: HashMap<&str, &str> = HashMap::new();
    //for (k, v) in args.arg_key.iter().zip(args.arg_val.iter()) {
    //    withs.insert(k, v);
    //}
    //let text = template::untemplate(&args.arg_file, &withs, true);
    //println!("untemplated: {:?}", text);
}

fn spag_request_list(args: &Args) {
    let filenames = try_error!(file::walk_dir(&dir));
    let mut yaml_files: Vec<&PathBuf> = filenames.iter()
        .filter(|p| p.to_str().unwrap().ends_with(".yml"))
        .collect();
    yaml_files.sort();
    for file in yaml_files.iter() {
        println!("{:?}", file);
    }
}

fn spag_method(args: &Args) {
    let method = get_method_from_args(args);
    let endpoint = match get_endpoint(args) {
        Ok(e) => { e },
        Err(msg) => { println!("{}", msg); return; }
    };
    let uri = args.arg_resource.to_string();
    let mut req = SpagRequest::new(method, endpoint, uri);
    req.add_headers(args.flag_header.iter());
    req.set_body(args.flag_data.clone());
    do_request(&req);
}

fn do_request(req: &SpagRequest) {
    // println!("{:?}", req);
    let mut handle = http::handle();
    let resp = req.prepare(&mut handle).exec().unwrap();
    // println!("{}", resp);
    println!("{}", String::from_utf8(resp.get_body().to_vec()).unwrap());
}

fn get_method_from_args(args: &Args) -> Method {
    if args.cmd_get { Method::Get }
    else if args.cmd_post { Method::Post }
    else if args.cmd_put { Method::Put }
    else if args.cmd_patch { Method::Patch }
    else if args.cmd_delete { Method::Delete }
    else { panic!("BUG: method not recognized"); }
}

fn get_endpoint(args: &Args) -> Result<String, String> {
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

fn get_dir(args: &Args) -> Result<String, String> {
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

fn get_string_from_yaml(y: &Yaml, keys: &[&str]) -> String {
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
