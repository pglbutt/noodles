use super::env;
use super::file;
use yaml_rust::YamlLoader;

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

#[test] fn test_ensure_extension() {
    assert!(&file::ensure_extension("aaa", "yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.", "yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa", ".yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.", ".yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.yml", "yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.yml", ".yml") == "aaa.yml");
    assert!(&file::ensure_extension("aaa.poo.", ".yml") == "aaa.poo.yml");
}
