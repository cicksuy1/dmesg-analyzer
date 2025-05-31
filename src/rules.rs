// src/rules.rs
use std::env;
use std::path::Path;
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

pub fn load_rules_with_fallback(path: Option<&str>, embedded: &str) -> RuleSet {
    // 1. Try CLI path
    if let Some(p) = path {
        if Path::new(p).exists() {
            if let Ok(contents) = std::fs::read_to_string(p) {
                if let Ok(rules) = toml::from_str(&contents) {
                    return rules;
                }
            }
        }
    }
    // 2. Try XDG config
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        let mut config_path = std::path::PathBuf::from(xdg);
        config_path.push("dmesg-analyzer/default_rules.toml");
        if config_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(rules) = toml::from_str(&contents) {
                    return rules;
                }
            }
        }
    }
    // 3. Try /usr/share
    let share_path = "/usr/share/dmesg-analyzer/default_rules.toml";
    if Path::new(share_path).exists() {
        if let Ok(contents) = std::fs::read_to_string(share_path) {
            if let Ok(rules) = toml::from_str(&contents) {
                return rules;
            }
        }
    }
    // 4. Fallback to embedded
    toml::from_str(embedded).expect("Could not parse embedded TOML rules")
}
