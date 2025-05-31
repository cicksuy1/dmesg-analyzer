// src/formatter.rs
use crate::rules::Rule;
use colored::*;

/// Formats a log line with the specified color and icon.
///
/// The color is applied using the `colored` crate. If the color is not recognized, the line is returned unstyled.
///
/// # Arguments
/// * `original_line` - The log line to format.
/// * `color_name` - The name of the color to apply (e.g., "red", "bold red").
/// * `icon` - The icon to prepend to the line.
///
/// # Returns
/// A formatted string with the icon and colored log line.
pub fn format_line(original_line: &str, color_name: &str, icon: &str) -> String {
    let colored_line = match color_name.to_lowercase().as_str() {
        "red" => original_line.red(),
        "bold red" => original_line.red().bold(),
        "green" => original_line.green(),
        "yellow" => original_line.yellow(),
        "blue" => original_line.blue(),
        "magenta" => original_line.magenta(),
        "cyan" => original_line.cyan(),
        "white" => original_line.white(),
        "black" => original_line.black(),
        // Add more colors as supported by the `colored` crate.
        _ => original_line.normal(), // Default if color is unrecognized.
    };
    format!("{} {}", icon, colored_line)
}
