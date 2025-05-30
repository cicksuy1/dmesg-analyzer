// src/formatter.rs
use crate::rules::Rule;
use colored::*;

pub fn format_line(line: &str, rule: &Rule) -> String {
    let colorized = match rule.color.as_str() {
        "red" => line.red(),
        "yellow" => line.yellow(),
        "green" => line.green(),
        "bold red" => line.red().bold(),
        _ => line.normal(),
    };
    format!("[{}] {}", rule.icon, colorized)
}
