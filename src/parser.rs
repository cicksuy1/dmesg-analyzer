// src/parser.rs
use crate::formatter::format_line;
use crate::rules::{LogCategory, Rule, RuleSet};

/// Parses a log line and categorizes it according to the provided rules.
///
/// Returns a tuple of the formatted string and its log category if a rule matches, or None otherwise.
pub fn parse_log(line: &str, rules: &RuleSet) -> Option<(String, LogCategory)> {
    // The order of checks determines priority: critical > error > warning > info.
    if matches_rule(line, &rules.critical) {
        Some((
            format_line(line, &rules.critical.color, &rules.critical.icon),
            LogCategory::Critical,
        ))
    } else if matches_rule(line, &rules.error) {
        Some((
            format_line(line, &rules.error.color, &rules.error.icon),
            LogCategory::Error,
        ))
    } else if matches_rule(line, &rules.warning) {
        Some((
            format_line(line, &rules.warning.color, &rules.warning.icon),
            LogCategory::Warning,
        ))
    } else if matches_rule(line, &rules.info) {
        // Optionally match info-specific keywords.
        Some((
            format_line(line, &rules.info.color, &rules.info.icon),
            LogCategory::Info,
        ))
    } else {
        // No rule matched; let the caller decide how to handle the line.
        None
    }
}

/// Checks if a log line matches any of the keywords in the given rule (case-insensitive).
fn matches_rule(line: &str, rule: &Rule) -> bool {
    // If no keywords are defined, this rule never matches.
    if rule.keywords.is_empty() {
        return false;
    }
    rule.keywords
        .iter()
        .any(|kw| line.to_lowercase().contains(&kw.to_lowercase()))
}
