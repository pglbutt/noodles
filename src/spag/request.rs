extern crate curl;

use std;
use std::ascii::AsciiExt;
use std::collections::hash_map::HashMap;
use curl::http;
use curl::http::handle::Method;

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
        Method::Options => "Options",
        Method::Get => "Get",
        Method::Head => "Head",
        Method::Post => "Post",
        Method::Put => "Put",
        Method::Patch => "Patch",
        Method::Delete => "Delete",
        Method::Trace => "Trace",
        Method::Connect => "Connect",
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
    /// Attempts to split on the ':' and panics if this fails.
    pub fn add_headers<'a, I: Iterator<Item=&'a String>>(&mut self, headers: I) {
        for header in headers {
            match http::header::parse(header.clone().into_bytes().as_slice()) {
                Some((name, val)) => { self.headers.insert(name.to_string(), val.to_string()); }
                None => { panic!(format!("Invalid header {:?}", header)); }
            }
        }
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
