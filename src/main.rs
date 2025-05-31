mod formatter;
mod parser;
mod rules;

use clap::Parser;
use inquire::Select;
use parser::parse_log;
use rules::{LogCategory, RuleSet, load_rules_with_fallback};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

const EMBEDDED_DEFAULT_RULES: &str = include_str!("../rules/default_rules.toml");

/// Command-line interface options for dmesg-analyzer.
///
/// Allows specifying a log file and a custom rules file.
#[derive(Parser)]
#[command(
    name = "dmesg-analyzer",
    version = "0.1.0",
    author = "Ben <benbald21@gmail.com>",
    about = "Highlight and summarize dmesg logs with colors and rules",
    long_about = "Reads kernel logs from dmesg or from a provided file and allows viewing categorized logs interactively."
)]
struct Cli {
    /// Analyze a dmesg log file instead of reading the current kernel log
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<String>,

    /// Path to a custom rule file (TOML format).
    /// If not provided, will search XDG config, /usr/share, or use embedded defaults.
    #[arg(short = 'R', long = "rules", value_name = "CUSTOM_RULES_PATH")]
    custom_rules_path: Option<String>,
}

/// Entry point for the dmesg-analyzer application.
///
/// Handles argument parsing, rule loading, log reading, parsing, and interactive display.
fn main() {
    let cli = Cli::parse();

    // Load rules from the specified path, XDG config, /usr/share, or fallback to embedded defaults.
    let (ruleset_instance, rules_source_info) =
        load_rules_with_fallback(cli.custom_rules_path.as_deref(), EMBEDDED_DEFAULT_RULES);

    if cli.custom_rules_path.is_some() {
        println!("Using custom rules from: {}", rules_source_info);
    } else {
        println!(
            "Using rules from: {} (Searched XDG, /usr/share, then embedded if not found)",
            rules_source_info
        );
    }

    // Step 1: Open dmesg source (file or live)
    let input: Box<dyn BufRead> = match cli.file {
        Some(ref path) => match File::open(path) {
            Ok(file) => Box::new(BufReader::new(file)),
            Err(e) => {
                eprintln!("Error: Failed to open log file '{}': {}", path, e);
                std::process::exit(1);
            }
        },
        None => {
            // If no file is provided, run `dmesg` and capture its output to a temporary file.
            let tmpfile = match NamedTempFile::new() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error: Failed to create temporary file: {}", e);
                    std::process::exit(1);
                }
            };

            let tmpfile_path = tmpfile.path().to_path_buf();

            match Command::new("dmesg")
                .stdout(
                    tmpfile
                        .as_file()
                        .try_clone()
                        .expect("Failed to clone temp file handle for dmesg"),
                )
                .status()
            {
                Ok(status) if status.success() => {}
                Ok(status) => {
                    eprintln!("Error: dmesg command failed with status: {}", status);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Error: Failed to run dmesg command: {}", e);
                    std::process::exit(1);
                }
            }

            // Reopen the temporary file for reading.
            match File::open(&tmpfile_path) {
                Ok(file) => Box::new(BufReader::new(file)),
                Err(e) => {
                    eprintln!(
                        "Error: Failed to reopen temporary dmesg file '{:?}': {}",
                        tmpfile_path, e
                    );
                    std::process::exit(1);
                }
            }
        }
    };

    // Step 2: Parse lines into categorized buckets.
    let mut critical_lines = Vec::new();
    let mut error_lines = Vec::new();
    let mut warning_lines = Vec::new();
    let mut info_lines = Vec::new();

    for line_result in input.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Warning: Skipping line due to read error: {}", e);
                continue;
            }
        };

        if let Some((formatted_string, category)) = parse_log(&line, &ruleset_instance) {
            match category {
                LogCategory::Critical => critical_lines.push(formatted_string),
                LogCategory::Error => error_lines.push(formatted_string),
                LogCategory::Warning => warning_lines.push(formatted_string),
                LogCategory::Info => info_lines.push(formatted_string),
            }
        } else {
            // If the line does not match any rule, add it as-is to info_lines.
            info_lines.push(line);
        }
    }

    // Step 3: Show interactive selection menu.
    loop {
        display_menu(&critical_lines, &error_lines, &warning_lines, &info_lines);
    }
}

/// Displays an interactive menu for selecting and viewing categorized log sections.
///
/// Uses the `inquire` crate for selection and `less` for paginated display.
fn display_menu(
    critical_lines: &[String],
    error_lines: &[String],
    warning_lines: &[String],
    info_lines: &[String],
) {
    println!("\n✔ Choose a section to view:\n");
    let options = vec![
        format!("🔥 Criticals ({})", critical_lines.len()),
        format!("❌ Errors ({})", error_lines.len()),
        format!("⚠️  Warnings ({})", warning_lines.len()),
        format!("ℹ️  Info ({})", info_lines.len()),
        "🚪 Exit".to_string(),
    ];

    let selection = Select::new("Section:", options)
        .with_help_message("Use arrows ↑↓ and press Enter. Press Esc to quit.")
        .prompt();

    match selection {
        Ok(choice) => {
            let lines_to_display = match choice.as_str() {
                s if s.starts_with("🔥") => critical_lines,
                s if s.starts_with("❌") => error_lines,
                s if s.starts_with("⚠️") => warning_lines,
                s if s.starts_with("ℹ️") => info_lines,
                s if s.starts_with("🚪") => {
                    println!("Exiting...");
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Internal error: Invalid selection received.");
                    return;
                }
            };

            if lines_to_display.is_empty() {
                println!("No logs in this section.");
                return;
            }

            let content = lines_to_display.join("\n");

            let pager_process = Command::new("less")
                .arg("-R")
                .arg("-F")
                .arg("-X")
                .stdin(Stdio::piped())
                .spawn();

            match pager_process {
                Ok(mut child) => {
                    if let Some(mut stdin) = child.stdin.take() {
                        if let Err(e) = stdin.write_all(content.as_bytes()) {
                            eprintln!("Error: Failed to write to pager stdin: {}", e);
                        }
                    }
                    if let Err(e) = child.wait() {
                        eprintln!("Error: Pager command failed to wait: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error: Failed to launch pager 'less': {}. Please ensure it's installed.",
                        e
                    );
                    println!("--- Displaying content directly as 'less' is unavailable ---");
                    println!("{}", content);
                    println!("--- End of content ---");
                }
            }
        }
        Err(_) => {
            println!("Exiting...");
            std::process::exit(0);
        }
    }
}
