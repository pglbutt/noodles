extern crate curl;
extern crate docopt;

use std;
use std::io::prelude::*;
use std::collections::hash_map::HashMap;
use std::path::PathBuf;

use curl::http;
use yaml_rust::Yaml;

use super::args;
use super::args::Args;
use super::env;
use super::file;
use super::history;
use super::remember;
use super::request;
use super::request::SpagRequest;
use super::template;
use super::yaml_util;


pub fn main() {
    let args: Args = args::parse_args();
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
        spag_env_deactivate();
    } else {
        error!("BUG: Invalid command");
    }
}

fn spag_env_set(args: &Args) {
    let use_shortcuts = true;
    let withs: HashMap<String, String> = args::get_withs(args);
    let withs: HashMap<&str, &str> = withs.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // untemplate all of the values
    let mut vals: Vec<String> = Vec::new();
    for v in &args.arg_val {
        let value = try_error!(template::untemplate(v.as_str(), &withs, use_shortcuts));
        vals.push(value);
    }

    try_error!(env::set_in_environment(&args.arg_environment, &args.arg_key, &vals));
    try_error!(env::show_environment(&args.arg_environment));
}

fn spag_env_unset(args: &Args) {
    if !args.flag_everything {
        try_error!(env::unset_in_environment(&args.arg_environment, &args.arg_key));
        try_error!(env::show_environment(&args.arg_environment));
    } else {
        try_error!(env::unset_all_environment(&args.arg_environment));
        try_error!(env::show_environment(&args.arg_environment));
    }
}

fn spag_env_show(args: &Args) {
    try_error!(env::show_environment(&args.arg_environment));
}

fn spag_env_activate(args: &Args) {
    try_error!(env::set_active_environment(&args.arg_environment));
    try_error!(env::show_environment(&args.arg_environment));
}

fn spag_env_deactivate() {
    env::deactivate_environment();
}

fn spag_history(args: &Args) {
    if args.cmd_show {
        spag_history_show(&args);
    } else {
        let short = try_error!(history::list());
        let mut count = 0;
        for line in short.iter() {
            println!("{}: {}", count, line);
            count += 1;
        }
    }
}

fn spag_history_show(args: &Args) {
    let out = try_error!(history::get(&args.arg_index));
    println!("{}", out);
}

fn spag_request(args: &Args) {
    if args.cmd_list {
        spag_request_list(args);
    } else if args.cmd_show {
        spag_request_show(args);
    } else {
        spag_request_a_file(args);
    }
}

fn spag_request_a_file(args: &Args) {
    let endpoint = try_error!(args::get_endpoint(args));
    let dir = try_error!(args::get_dir(args));
    let withs: HashMap<String, String> = args::get_withs(args);
    let withs: HashMap<&str, &str> = withs.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // load the request file, but untemplate it first
    let request_filename = try_error!(request::get_request_filename(&args.arg_file, &dir));
    let yaml_string = try_error!(file::read_file(&request_filename));
    let use_shortcuts = false;
    let yaml_string = try_error!(template::untemplate(&yaml_string, &withs, use_shortcuts));

    match yaml_util::load_yaml_string(&yaml_string) {
        Ok(y) => {
            let method = try_error!(yaml_util::get_value_as_string(&y, &["method"]));
            let uri = try_error!(yaml_util::get_value_as_string(&y, &["uri"]));

            // the request body can be overridden by the --data flag.
            //
            // todo? because docopt defaults to an empty string if the data flag isn't given,
            // we can't tell if the user is trying to override the body to be empty.
            let data = try_error!(args::get_data(args));
            let body =
                if !data.is_empty() {
                    data
                } else {
                    if let Some(&Yaml::String(ref b)) = yaml_util::get_nested_value(&y, &["body"]) {
                        b.to_string()
                    } else {
                        String::new()
                    }
                };

            let headers = try_error!(args::resolve_headers(args, &y));

            let mut req = SpagRequest::new(request::method_from_str(&method), endpoint, uri);
            try_error!(req.add_headers(headers.iter()));
            req.set_body(body);
            do_request(&req);
        },
        Err(msg) => { error!("{}", msg); }
    }
}

fn spag_request_show(args: &Args) {
    let dir = try_error!(args::get_dir(args));
    let filename = try_error!(request::get_request_filename(&args.arg_file, &dir));
    let contents = try_error!(file::read_file(&filename));
    println!("{}", contents);
}

fn spag_request_list(args: &Args) {
    let dir = try_error!(args::get_dir(args));
    let filenames = try_error!(file::walk_dir(&dir));
    let mut yaml_files: Vec<&PathBuf> = filenames.iter()
        .filter(|p| p.to_str().unwrap().ends_with(".yml"))
        .collect();
    yaml_files.sort();

    let current_dir = try_error!(std::env::current_dir());
    for file in yaml_files.iter() {
        // relative_from() is unstable
        println!("{}", file.relative_from(&current_dir).unwrap().to_str().unwrap());
    }
}

fn spag_method(args: &Args) {
    // untemplate the resource
    let use_shortcuts = true;
    let withs: HashMap<String, String> = args::get_withs(args);
    let withs: HashMap<&str, &str> = withs.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let resource = try_error!(template::untemplate(&args.arg_resource, &withs, use_shortcuts));

    let method = args::get_method_from_args(args);
    let endpoint = try_error!(args::get_endpoint(args));
    let mut req = SpagRequest::new(method, endpoint, resource);
    let headers = try_error!(args::resolve_headers_no_request_file(args));
    try_error!(req.add_headers(headers.iter()));

    let body = try_error!(args::get_data(args));
    req.set_body(body);
    do_request(&req);
}

fn do_request(req: &SpagRequest) {
    let mut handle = http::handle();
    let resp = try_error!(req.prepare(&mut handle).exec());
    println!("{}", String::from_utf8(resp.get_body().to_vec()).unwrap());
    try_error!(history::append(req, &resp));
    try_error!(remember::remember(req, &resp));
}

