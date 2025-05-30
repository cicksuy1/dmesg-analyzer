// src/rules.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RuleSet {
    pub error: Rule,
    pub warning: Rule,
    pub critical: Rule,
    pub info: Rule,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub keywords: Vec<String>,
    pub color: String,
    pub icon: String,
}

pub fn load_rules(path: &str) -> RuleSet {
    let contents = std::fs::read_to_string(path).expect("Could not read rules file");
    toml::from_str(&contents).expect("Could not parse TOML rules")
}
