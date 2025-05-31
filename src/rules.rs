use serde::Deserialize;
use std::env;
use std::path::Path;

/// Represents the log category for a parsed log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    /// Critical severity log
    Critical,
    /// Error severity log
    Error,
    /// Warning severity log
    Warning,
    /// Informational log
    Info,
}

/// A set of rules for categorizing log lines by severity.
#[derive(Debug, Deserialize)]
pub struct RuleSet {
    /// Rules for critical logs
    pub critical: Rule,
    /// Rules for error logs
    pub error: Rule,
    /// Rules for warning logs
    pub warning: Rule,
    /// Rules for informational logs
    pub info: Rule,
}

/// A rule for matching log lines, including keywords, color, and icon.
#[derive(Debug, Deserialize)]
pub struct Rule {
    /// Keywords that trigger this rule
    pub keywords: Vec<String>,
    /// Color name for highlighting
    pub color: String,
    /// Icon to display with the log
    pub icon: String,
}

/// Loads rules from a custom path, XDG config, /usr/share, or falls back to embedded defaults.
///
/// Returns a tuple of the loaded RuleSet and a string describing the source.
///
/// # Arguments
/// * `cli_path` - Optional path to a custom rules file.
/// * `embedded_rules_str` - Embedded TOML rules as a string.
///
/// # Panics
/// Panics if the embedded rules are invalid.
pub fn load_rules_with_fallback(
    cli_path: Option<&str>,
    embedded_rules_str: &str,
) -> (RuleSet, String) {
    // 1. Try CLI path
    if let Some(p_str) = cli_path {
        let p = Path::new(p_str);
        if p.exists() {
            if let Ok(contents) = std::fs::read_to_string(p) {
                if let Ok(rules) = toml::from_str(&contents) {
                    return (rules, p_str.to_string());
                } else {
                    eprintln!(
                        "Warning: Failed to parse custom rules file '{}'. Falling back.",
                        p_str
                    );
                }
            } else {
                eprintln!(
                    "Warning: Failed to read custom rules file '{}'. Falling back.",
                    p_str
                );
            }
        } else {
            eprintln!(
                "Warning: Custom rules file '{}' not found. Falling back.",
                p_str
            );
        }
    }

    // 2. Try XDG config directory
    if let Some(xdg_os_str) = env::var_os("XDG_CONFIG_HOME") {
        let mut config_path = std::path::PathBuf::from(xdg_os_str);
        config_path.push("dmesg-analyzer/default_rules.toml");
        if config_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(rules) = toml::from_str(&contents) {
                    return (rules, config_path.to_string_lossy().into_owned());
                } else {
                    eprintln!(
                        "Warning: Failed to parse XDG rules file {:?}. Falling back.",
                        config_path
                    );
                }
            } else {
                eprintln!(
                    "Warning: Failed to read XDG rules file {:?}. Falling back.",
                    config_path
                );
            }
        }
    }

    // 3. Try /usr/share system directory
    let share_path_str = "/usr/share/dmesg-analyzer/default_rules.toml";
    let share_path = Path::new(share_path_str);
    if share_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(share_path) {
            if let Ok(rules) = toml::from_str(&contents) {
                return (rules, share_path_str.to_string());
            } else {
                eprintln!(
                    "Warning: Failed to parse system rules file '{}'. Falling back.",
                    share_path_str
                );
            }
        } else {
            eprintln!(
                "Warning: Failed to read system rules file '{}'. Falling back.",
                share_path_str
            );
        }
    }

    // 4. Fallback to embedded rules (should always succeed)
    match toml::from_str(embedded_rules_str) {
        Ok(rules) => (rules, "embedded".to_string()),
        Err(e) => {
            panic!("Could not parse embedded TOML rules: {}", e);
        }
    }
}
