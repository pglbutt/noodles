extern crate curl;

use std;
use std::ascii::AsciiExt;
use std::collections::hash_map::HashMap;

use curl::http;
use curl::http::handle::Method;
use yaml_rust::Yaml;
use super::file;


/// Split "Content-type: application/json" into vec!["Content-type", "application/json"]
pub fn split_header<'a>(header: &'a str) -> Result<(&'a str, &'a str), String> {
    // Strip out any spaces and separate the components
    let h: Vec<&str> = header.split(":").map(|s| s.trim()).collect();

    // If the header didn't have a colon, or had more than 1, it's invalid
    if h.len() != 2 {
        Err(format!("Invalid header {:?}", header))
    } else {
        Ok((h[0], h[1]))
    }
}

pub fn get_request_filename(name: &str, dir: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("No request filename given".to_string());
    }
    let options = try!(file::find_matching_files(&file::ensure_extension(name, "yml"), dir));
    if options.is_empty() {
        Err(format!("Request file {:?} not found", name))
    } else if options.len() == 1 {
        Ok(options[0].to_str().unwrap().to_string())
    } else {
        Err(format!("Ambiguous request name. Choose one of {:?}", options))
    }
}

pub fn load_request_file(name: &str, dir: &str) -> Result<Yaml, String> {
    let filename = try!(get_request_filename(name, dir));
    file::load_yaml_file(&filename)
}

pub fn method_from_str(s: &str) -> Method {
    match s.to_ascii_lowercase().as_str() {
        "options" => Method::Options,
        "get" => Method::Get,
        "head" => Method::Head,
        "post" => Method::Post,
        "put" => Method::Put,
        "patch" => Method::Patch,
        "delete" => Method::Delete,
        "trace" => Method::Trace,
        "connect" => Method::Connect,
        _ => { panic!(format!("Invalid method string {}", s)); }
    }
}

pub fn method_to_str(m: Method) -> &'static str {
    match m {
        Method::Options => "OPTIONS",
        Method::Get => "GET",
        Method::Head => "HEAD",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Patch => "PATCH",
        Method::Delete => "DELETE",
        Method::Trace => "TRACE",
        Method::Connect => "CONNECT",
    }
}

pub struct SpagRequest {
    pub method: Method,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub endpoint: String,
    pub uri: String,
}

impl SpagRequest {

    pub fn new(method: Method, endpoint: String, uri: String) -> SpagRequest {
        SpagRequest {
            method: method,
            body: String::new(),
            headers: HashMap::new(),
            endpoint: endpoint,
            uri: uri
        }
    }

    pub fn get_method_string(&self) -> &'static str {
        method_to_str(self.method)
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Headers is some iterable of Strings like "Content-type: application/json".
    /// Attempts to split on the ':' and returns a Result on failure
    pub fn add_headers<'a, I: Iterator<Item=&'a String>>(&mut self, headers: I) -> Result<(), String> {
        for rawheader in headers {
            let h = try!(split_header(rawheader));
            let name =
                if h.0.to_lowercase() == "content-type" {
                    "Content-Type".to_string()
                } else {
                    h.0.to_string()
                };
            let value = h.1.to_string();

            self.headers.insert(name, value);
        }
        Ok(())
    }

    pub fn prepare<'a, 'b>(&'b self, handle: &'a mut http::Handle) -> http::Request<'a, 'b> {
        let uri = self.endpoint.to_string() + &self.uri;
        let headers = self.headers.iter().map(|(a, b)| (a.as_str(), b.as_str()));
        http::Request::new(handle, self.method)
            .uri(uri.to_string())
            .headers(headers)
            .body(&self.body)
    }
}

impl std::fmt::Debug for SpagRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SpagRequest")
            .field("method", &self.get_method_string())
            .field("headers", &self.headers)
            .field("endpoint", &self.endpoint)
            .field("uri", &self.uri)
            .field("body", &self.body)
            .finish()
    }
}
