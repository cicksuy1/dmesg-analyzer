// src/parser.rs
use crate::formatter::format_line;
use crate::rules::{Rule, RuleSet};

pub fn parse_log(line: &str, rules: &RuleSet) -> Option<String> {
    if matches_rule(line, &rules.critical) {
        Some(format_line(line, &rules.critical))
    } else if matches_rule(line, &rules.error) {
        Some(format_line(line, &rules.error))
    } else if matches_rule(line, &rules.warning) {
        Some(format_line(line, &rules.warning))
    } else if matches_rule(line, &rules.info) {
        Some(format_line(line, &rules.info))
    } else {
        None
    }
}

fn matches_rule(line: &str, rule: &Rule) -> bool {
    rule.keywords
        .iter()
        .any(|kw| line.to_lowercase().contains(&kw.to_lowercase()))
}
