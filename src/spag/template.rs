use super::env;
use std::collections::hash_map::HashMap;
use yaml_rust::Yaml;

const VALID_ITEM_NAME_CHARS: &'static str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_";

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Substitute(Vec<Token<'a>>),
    Text(&'a str),
    With(&'a str),
    Env(&'a str, Vec<&'a str>),
    Request(&'a str, Vec<&'a str>),
    DefaultVal(&'a str),
}

/// Untemplate the given string
///
/// Strategy:
///     "aaaa{{thing, [env].thing.id, last.response.body.thing}}"
///
/// Becomes, roughly:
///     vec![
///         Text("aaaa"),
///         Substitute([
///             With("aaaa"),
///             Env("env", ["thing", "id"]),
///             Request("last", ["response", "body", "thing"]),
///         ])
///     ]
pub fn untemplate(text: &str, withs: &HashMap<&str, &str>, shortcuts: bool
                  ) -> Result<String, String> {
    let tokens = try!(Tokenizer::new(text, shortcuts).tokenize());
    let mut result = String::new();
    for token in tokens {
        match token {
            Token::Text(text) => {
                result.push_str(text);
            },
            Token::Substitute(options) => {
                let text = try!(substitute(&options, &withs));
                result.push_str(&text);
            },
            // todo: this would be solved in the type system by breaking up the Token enum into
            //    enum Token { Substitute, Text }
            //    enum SubstituteItem { With, Env, Request, DefaultVal }
            _ => { return Err(format!("BUG: invalid top-level Token option")); }
        }
    }
    Ok(result)
}

fn substitute<'a>(options: &Vec<Token<'a>>, withs: &HashMap<&str, &str>
                  ) -> Result<String, String> {
    for option in options {
        match option {
            &Token::With(with) => {
                if let Some(val) = withs.get(with) {
                    return Ok(val.to_string());
                }
            },
            &Token::Env(name, ref key_path) => {
                if let Ok(y) = env::load_environment(name) {
                    if let Some(&Yaml::String(ref val)) = env::get_nested_value(&y, &key_path) {
                        return Ok(val.to_string());
                    }
                }
            },
            &Token::Request(name, ref key_path) => {
                return Err("substitution not implemented for requests".to_string());
            },
            &Token::DefaultVal(val) => {
                return Ok(val.to_string());
            },
            _ => { return Err("BUG: Saw invalid enum option in substitute".to_string()); },
        }
    }
    // TODO: better error message here
    Err("substitution failed".to_string())
}

pub struct Tokenizer<'a> {
    text: &'a str,
    char_indices: Vec<(usize, char)>,
    i: usize,
    shortcuts: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(text: &'a str, shortcuts: bool) -> Tokenizer<'a> {
        Tokenizer {
            text: text,
            char_indices: text.char_indices().collect(),
            i: 0,
            shortcuts: shortcuts,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token<'a>>, String> {
        self.i = 0;
        let mut result: Vec<Token<'a>> = Vec::new();
        loop {
            if self.eof() {
                break;
            } else if self.has("{{") {
                result.push(try!(self.read_braces()));
            } else if self.shortcuts && self.has("@") {
                self.i += 1;
                let token = try!(self.read_brace_item());
                result.push(Token::Substitute(vec![token]));
            } else {
                result.push(try!(self.read_text()));
            }
        }
        Ok(result)
    }

    /// Use this to read any text outside of {{...}} lists and the shortcut notation @<thing>
    fn read_text(&mut self) -> Result<Token<'a>, String> {
        if self.eof() { return Err("Expected text?".to_string()); }
        let (start, _) = self.char_indices[self.i];
        let mut end;
        loop {
            if self.eof() {
                end = self.text.len();
                break;
            }
            let (offset, _) = self.char_indices[self.i];
            if self.has("{{") || (self.shortcuts && self.has("@")) {
                end = offset;
                break;
            }
            self.i += 1;
        }
        if start == end {
            Err("Failed to read text".to_string())
        } else {
            Ok(Token::Text(&self.text[start..end]))
        }
    }

    /// Read a list of items delimited by double braces, like {{<item>, <item>, ...}}
    fn read_braces(&mut self) -> Result<Token<'a>, String> {
        if !self.has("{{") { panic!("BUG: read_braces called when no braces found"); }
        self.i += 2;
        let mut result: Vec<Token<'a>> = Vec::new();
        loop {
            let token = try!(self.read_brace_item());
            result.push(token);
            self.skip_whitespace();
            if self.has("}}") {
                self.i += 2;
                break;
            } else if self.eof() {
                return Err("Unclosed braces".to_string());
            } else if self.has(",") {
                self.i += 1;
            } else if self.has(":") {
                self.i += 1;
                let default = try!(self.read_default_value()).trim();
                if !self.has("}}") {
                    return Err("Default value must be the last list item after the ':'.".to_string());
                }
                result.push(Token::DefaultVal(default));
                self.i += 2;
                break;
            }
        }
        if result.is_empty() {
            Err("Found empty braces".to_string())
        } else {
            Ok(Token::Substitute(result))
        }
    }

    /// Read one of:
    ///     - A --with key like <name>
    ///     - A request lookup like "last.response.body.id"
    ///     - An environment lookup like "[env].thing.id"
    fn read_brace_item(&mut self) -> Result<Token<'a>, String> {
        self.skip_whitespace();
        if self.eof() {
            return Err("Empty item".to_string());
        } else if self.has("[") {
            self.read_env_item()
        } else {
            self.read_with_or_request_item()
        }
    }

    /// Read something like [<name>].<key>.<key>
    fn read_env_item(&mut self) -> Result<Token<'a>, String> {
        assert!(!self.eof());

        try!(self.expect_char('['));
        self.skip_whitespace();
        let env_name =
            // an empty environment name means use the active env
            if self.has("]") {
                self.i += 1;
                ""
            } else {
                let name = try!(self.read_item_name());
                self.skip_whitespace();
                try!(self.expect_char(']'));
                name
            };

        let key_path = try!(self.read_key_path());

        if key_path.is_empty() {
            Err(format!("No key found after environment {}", env_name))
        } else {
            Ok(Token::Env(env_name, key_path))
        }
    }

    /// Read a --with key name that looks like <name> -- returns a Token::With
    /// Or read a request name + key_path, like "last.response.body.id" -- returns a Token::Request
    fn read_with_or_request_item(&mut self) -> Result<Token<'a>, String> {
        assert!(!self.eof());
        let name = try!(self.read_item_name());
        if self.has(".") {
            let key_path = try!(self.read_key_path());
            if key_path.is_empty() {
                Err(format!("Expected key after \"{}.\"", name))
            } else {
                Ok(Token::Request(name, key_path))
            }
        } else {
            Ok(Token::With(name))
        }
    }

    /// Read a key path, starting with a '.', like ".response.body.id"
    fn read_key_path(&mut self) -> Result<Vec<&'a str>, String> {
        let mut key_path: Vec<&'a str> = Vec::new();
        loop {
            if self.eof() { break; }
            if !self.has(".") {
                break;
            } else {
                self.i += 1;
            }
            let part = try!(self.read_item_name());
            key_path.push(part);
        }
        Ok(key_path)
    }

    /// Return the text between the ':' and closing "}}" in a list.
    /// Assumes the ':' has already been consumed.
    fn read_default_value(&mut self) -> Result<&'a str, String> {
        self.skip_whitespace();
        if self.eof() {
            return Err("Found eof while reading default value. Unclosed braces".to_string());
        }
        let (start, _) = self.char_indices[self.i];
        let mut end;
        loop {
            if self.eof() {
                return Err("Found eof while reading default value. Unclosed braces".to_string());
            }
            if self.has("}}") {
                let (offset, _) = self.char_indices[self.i];
                end = offset;
                break;
            }
            self.i += 1;
        }
        if start == end {
            Err("Found empty default value.".to_string())
        } else {
            Ok(&self.text[start..end])
        }
    }

    /// Read contiguous text containing characters in VALID_ITEM_NAME_CHARS
    fn read_item_name(&mut self) -> Result<&'a str, String> {
        if self.eof() {
            return Err("Expected a name but found eof".to_string());
        }
        let (start, _) = self.char_indices[self.i];
        let mut end;
        loop {
            if self.eof() {
                end = self.text.len();
                break;
            }
            let (offset, c) = self.char_indices[self.i];
            if !VALID_ITEM_NAME_CHARS.contains(c) {
                end = offset;
                break;
            }
            self.i += 1;
        }
        if start == end {
            Err("No item found".to_string())
        } else {
            Ok(&self.text[start..end])
        }
    }

    /// Check if the current position starts with the given string
    fn has(&self, s: &'static str) -> bool {
        if self.eof() { return false; }
        let (offset, _) = self.char_indices[self.i];
        let end =
            if offset + s.len() > self.text.len() {
                self.text.len()
            } else {
                offset + s.len()
            };
        s == &self.text[offset .. end]
    }

    /// Check for the end of the text
    fn eof(&self) -> bool {
        self.i >= self.char_indices.len()
    }

    /// Return but do not consume the char at the current position
    fn peek_char(&mut self) -> Option<char> {
        if self.eof() {
            None
        } else {
            let (_, c) = self.char_indices[self.i];
            Some(c)
        }
    }

    /// Consume the char at the current position if it matches. Else error.
    fn expect_char(&mut self, expected: char) -> Result<char, String> {
        if self.eof() {
            return Err(format!("Expected character {:?} but found eof", expected));
        }
        let c = self.peek_char().unwrap();
        if c == expected {
            self.i += 1;
            Ok(c)
        } else {
            Err(format!("Expected {:?} but found {:?}", expected, c))
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek_char() {
                Some(c) if c.is_whitespace() => { self.i += 1; }
                _ => { break; }
            }
        }
    }
}
