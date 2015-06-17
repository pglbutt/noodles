use super::env;
use super::file;
use super::template;
use super::template::Token;
use yaml_rust::YamlLoader;
use std::collections::hash_map::HashMap;

#[test] fn test_set_nested_value_in_yaml() {
    let mut doc = &mut YamlLoader::load_from_str("{}").unwrap()[0];

    // check setting values in maps that don't exist
    env::set_nested_value(doc, &["mini"], "wumbo");
    env::set_nested_value(doc, &["a", "b", "c"], "ABC");
    assert!(doc["mini"].as_str().unwrap() == "wumbo");
    assert!(doc["a"]["b"]["c"].as_str().unwrap() == "ABC");

    // check overwriting existing entries
    env::set_nested_value(doc, &["mini"], "X");
    env::set_nested_value(doc, &["a", "b", "c"], "XYZ");
    assert!(doc["mini"].as_str().unwrap() == "X");
    assert!(doc["a"]["b"]["c"].as_str().unwrap() == "XYZ");
}

#[test] fn test_unset_nested_value_in_yaml() {
    // - foo: bar
    // - spongebob: squarepants
    // - headers:
    //      - content-type: application/json
    //      - accept: application/json";
    let mut doc = &mut YamlLoader::load_from_str("{}").unwrap()[0];
    env::set_nested_value(doc, &["foo"], "bar");
    env::set_nested_value(doc, &["spongebob"], "squarepants");
    env::set_nested_value(doc, &["headers", "content-type"], "application/json");
    env::set_nested_value(doc, &["headers", "accept"], "application/json");

    // check unsetting nested and unnested values
    env::unset_nested_value(doc, &["headers", "accept"]);
    env::unset_nested_value(doc, &["foo"]);
    // Access non-exist node by Index trait will return BadValue.
    assert!(doc["foo"].is_badvalue() == true);
    assert!(doc["headers"]["accept"].is_badvalue() == true);

    // Check the other values exist
    assert!(doc["spongebob"].as_str().unwrap() == "squarepants");
    assert!(doc["headers"]["content-type"].as_str().unwrap() == "application/json");
}

#[test] fn test_ensure_extension() {
    assert!(&file::ensure_extension("aaa", "yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.", "yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa", ".yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.", ".yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.yml", "yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.yml", ".yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.poo.", ".yml") == "aaa.poo.yml");
}

#[test] fn test_tokenize_text() {
    let tokens = template::Tokenizer::new("wumbo", true).tokenize().unwrap();
    assert_eq!(tokens, vec![Token::Text("wumbo")]);
}

#[test] fn test_tokenize_list() {
    let tokens = template::Tokenizer::new("{{wumbo, aaa.bbb : ccc }}", true).tokenize().unwrap();
    assert_eq!(tokens, vec![
        Token::Substitute(vec![
            Token::With("wumbo"),
            Token::Request("aaa", vec!["bbb"]),
            Token::DefaultVal("ccc"),
        ])]);
}

#[test] fn test_tokenize_shortcut() {
    let tokens = template::Tokenizer::new("@wumbo", true).tokenize().unwrap();
    assert_eq!(tokens, vec![ Token::Substitute(vec![ Token::With("wumbo") ])]);
}

#[test] fn test_tokenize_text_list_shortcut_together() {
    let text =
        "  pglbutt   @[ env ].wumbo.thing_1234567890/poo{{last.response.body.id}}\t\nhello \t";
    let tokens = template::Tokenizer::new(text, true).tokenize().unwrap();
    assert_eq!(tokens, vec![
        Token::Text("  pglbutt   "),
        Token::Substitute(vec![ Token::Env("env", vec!["wumbo", "thing_1234567890"]) ]),
        Token::Text("/poo"),
        Token::Substitute(vec![ Token::Request("last", vec!["response", "body", "id"]) ]),
        Token::Text("\t\nhello \t"),
    ]);
}

#[test] fn test_token_text_shortcuts_disabled() {
    let tokens = template::Tokenizer::new("@a{{b}}", false).tokenize().unwrap();
    assert_eq!(tokens, vec![
        Token::Text("@a"),
        Token::Substitute(vec![ Token::With("b") ]),
    ]);
}

#[test] fn test_untemplate_withs() {
    let mut withs: HashMap<&str, &str> = HashMap::new();
    withs.insert("a", "A");
    withs.insert("b", "B");
    let text = template::untemplate("@a{{b}}", &withs, true).unwrap();
    assert_eq!(&text, "AB");
    let text = template::untemplate("{{a}}{{b}}", &withs, true).unwrap();
    assert_eq!(&text, "AB");
    let text = template::untemplate("{{a}}@b", &withs, true).unwrap();
    assert_eq!(&text, "AB");
    let text = template::untemplate("@a@b", &withs, true).unwrap();
    assert_eq!(&text, "AB");
    let text = template::untemplate("  mini  {{a}}  @b  wumbo  ", &withs, true).unwrap();
    assert_eq!(&text, "  mini  A  B  wumbo  ");
}

#[test] fn test_untemplate_withs_w_many_items() {
    let mut withs: HashMap<&str, &str> = HashMap::new();
    withs.insert("a", "A");
    let text = template::untemplate("{{a, b, c}}", &withs, true).unwrap();
    assert_eq!(&text, "A");
    let text = template::untemplate("{{b, a, c}}", &withs, true).unwrap();
    assert_eq!(&text, "A");
    let text = template::untemplate("{{b, c, a}}", &withs, true).unwrap();
    assert_eq!(&text, "A");
}

#[test] fn test_untemplate_list_w_default_value() {
    let mut withs: HashMap<&str, &str> = HashMap::new();
    withs.insert("a", "A");
    let text = template::untemplate("{{a, b : hello}}", &withs, true).unwrap();
    assert_eq!(&text, "A");
    let text = template::untemplate("{{b, a : hello}}", &withs, true).unwrap();
    assert_eq!(&text, "A");
    let text = template::untemplate("{{b, c : hello}}", &withs, true).unwrap();
    assert_eq!(&text, "hello");
}

#[test] fn test_untemplate_list_no_substitute_found() {
    let withs: HashMap<&str, &str> = HashMap::new();
    let result = template::untemplate("{{a, b, c}}", &withs, true);
    assert!(result.is_err());
    let result = template::untemplate("@a", &withs, true);
    assert!(result.is_err());
}
