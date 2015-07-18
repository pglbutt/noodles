use std::collections::hash_map::HashMap;

use yaml_rust::YamlLoader;
use rustc_serialize::json::Json;

use super::file;
use super::template;
use super::template::{Token, Choice};
use super::remember;
use super::yaml_util;

#[test] fn test_set_nested_value_in_yaml() {
    let mut doc = &mut YamlLoader::load_from_str("{}").unwrap()[0];

    // check setting values in maps that don't exist
    yaml_util::set_nested_value(doc, &["mini"], "wumbo");
    yaml_util::set_nested_value(doc, &["a", "b", "c"], "ABC");
    assert!(doc["mini"].as_str().unwrap() == "wumbo");
    assert!(doc["a"]["b"]["c"].as_str().unwrap() == "ABC");

    // check overwriting existing entries
    yaml_util::set_nested_value(doc, &["mini"], "X");
    yaml_util::set_nested_value(doc, &["a", "b", "c"], "XYZ");
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
    yaml_util::set_nested_value(doc, &["foo"], "bar");
    yaml_util::set_nested_value(doc, &["spongebob"], "squarepants");
    yaml_util::set_nested_value(doc, &["headers", "content-type"], "application/json");
    yaml_util::set_nested_value(doc, &["headers", "accept"], "application/json");

    // check unsetting nested and unnested values
    yaml_util::unset_nested_value(doc, &["headers", "accept"]);
    yaml_util::unset_nested_value(doc, &["foo"]);
    // Access non-exist node by Index trait will return BadValue.
    assert!(doc["foo"].is_badvalue() == true);
    assert!(doc["headers"]["accept"].is_badvalue() == true);

    // Check the other values exist
    assert!(doc["spongebob"].as_str().unwrap() == "squarepants");
    assert!(doc["headers"]["content-type"].as_str().unwrap() == "application/json");
}

#[test] fn test_json_find_path() {
    let data = Json::from_str(r#"
        {"a":
            [{"b":
                {"c": "hello"}
            }]
        }
    "#).unwrap();
    assert!(remember::json_find_path(&data, &["a"]).unwrap().is_array());
    assert_eq!(Ok(&Json::String("hello".to_string())),
               remember::json_find_path(&data, &["a", "0", "b", "c"]));


    assert!(remember::json_find_path(&data, &["a", "1"]).is_err());
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
            Choice::With("wumbo"),
            Choice::Request("aaa", vec!["bbb".to_string()]),
            Choice::DefaultVal("ccc"),
        ])]);
}

#[test] fn test_tokenize_shortcut() {
    let key_path = vec!["response".to_string(), "body".to_string(), "wumbo".to_string()];

    let tokens = template::Tokenizer::new("@wumbo", true).tokenize().unwrap();
    assert_eq!(tokens, vec![ Token::Substitute(vec![ Choice::Request("last", key_path.clone()) ])]);

    let tokens = template::Tokenizer::new("@body.wumbo", true).tokenize().unwrap();
    assert_eq!(tokens, vec![ Token::Substitute(vec![ Choice::Request("last", key_path.clone()) ])]);

    let key_path = vec!["response".to_string(), "body".to_string(), "things".to_string(), "0".to_string(), "id".to_string()];
    let tokens = template::Tokenizer::new("@body.things.0.id", true).tokenize().unwrap();
    assert_eq!(tokens, vec![ Token::Substitute(vec![ Choice::Request("last", key_path) ])]);
}

#[test] fn test_tokenize_text_list_shortcut_together() {
    let text =
        "  pglbutt   @[ yaml_util ].wumbo.thing_1234567890/poo{{last.response.body.id}}\t\nhello \t";
    let tokens = template::Tokenizer::new(text, true).tokenize().unwrap();
    let key_path = vec!["response".to_string(), "body".to_string(), "id".to_string()];
    assert_eq!(tokens, vec![
        Token::Text("  pglbutt   "),
        Token::Substitute(vec![ Choice::Env("yaml_util", vec!["wumbo", "thing_1234567890"]) ]),
        Token::Text("/poo"),
        Token::Substitute(vec![ Choice::Request("last", key_path) ]),
        Token::Text("\t\nhello \t"),
    ]);
}

#[test] fn test_tokenize_text_shortcuts_disabled() {
    let tokens = template::Tokenizer::new("@a{{b}}", false).tokenize().unwrap();
    assert_eq!(tokens, vec![
        Token::Text("@a"),
        Token::Substitute(vec![ Choice::With("b") ]),
    ]);
}

#[test] fn test_untemplate_withs() {
    let mut withs: HashMap<&str, &str> = HashMap::new();
    withs.insert("a", "A");
    withs.insert("b", "B");
    let text = template::untemplate("{{a}}{{b}}", &withs, true).unwrap();
    assert_eq!(&text, "AB");
    let text = template::untemplate("  mini  {{ a }}  wumbo  ", &withs, true).unwrap();
    assert_eq!(&text, "  mini  A  wumbo  ");
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

#[test] fn test_show_params_for_choices() {
    let options = vec![
        Choice::With("with-key"),
        Choice::Env("", vec!["a", "b", "c"]),
        Choice::Env("myenv", vec!["c", "d", "e"]),
        Choice::Request("last", vec!["body".to_string(), "id".to_string()]),
        Choice::Request("other", vec!["headers".to_string(), "accept".to_string()]),
    ];

    let result = template::show_params_for_choices(&options).unwrap();
    let expected = concat!(
        "{{ with-key, [].a.b.c, [myenv].c.d.e, last.body.id, other.headers.accept }} needs one of\n",
        "    * flag \"--with with-key <value>\"\n",
        "    * key [\"a\", \"b\", \"c\"] from the active environment\n",
        "    * key [\"c\", \"d\", \"e\"] from environment \"myenv\"\n",
        "    * key [\"body\", \"id\"] from the previous request\n",
        "    * key [\"headers\", \"accept\"] from the request saved as \"other\"\n");
    assert_eq!(result.as_str(), expected);
}
