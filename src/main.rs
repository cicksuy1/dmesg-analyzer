mod formatter;
mod parser;
mod rules;

use clap::Parser;
use inquire::Select;
use parser::parse_log;
use rules::load_rules;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

#[derive(Parser)]
#[command(
    name = "dmesg-analyzer",
    version = "0.1.0",
    author = "Ben <you@example.com>",
    about = "Highlight and summarize dmesg logs with colors and rules",
    long_about = "Reads kernel logs from dmesg or from a provided file and allows viewing categorized logs interactively."
)]
struct Cli {
    /// Analyze a dmesg log file instead of reading the current kernel log
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<String>,

    /// Path to the rule file (TOML format)
    #[arg(
        short = 'r',
        long = "rules",
        value_name = "RULES",
        default_value = "rules/default_rules.toml"
    )]
    rules: String,
}

fn main() {
    let cli = Cli::parse();
    let rules_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), cli.rules);
    eprintln!("Using rules from: {}", rules_path);
    let rules = load_rules(&rules_path);

    // Step 1: Open dmesg source (file or live)
    let input: Box<dyn BufRead> = match cli.file {
        Some(ref path) => {
            let file = File::open(path).expect("Failed to open log file");
            Box::new(BufReader::new(file))
        }
        None => {
            let mut tmpfile = NamedTempFile::new().expect("Failed to create temporary file");

            {
                let mut file_handle = tmpfile.as_file_mut();
                let mut child = Command::new("dmesg")
                    .stdout(Stdio::from(
                        file_handle
                            .try_clone()
                            .expect("Failed to clone temp file handle"),
                    ))
                    .spawn()
                    .expect("Failed to run dmesg");

                let status = child.wait().expect("Failed to wait on dmesg");
                if !status.success() {
                    panic!("dmesg failed with status: {}", status);
                }
            }

            let file = File::open(tmpfile.path()).expect("Failed to reopen dmesg temp file");
            Box::new(BufReader::new(file))
        }
    };

    // Step 2: Parse lines into buckets
    let mut critical_lines = Vec::new();
    let mut error_lines = Vec::new();
    let mut warning_lines = Vec::new();
    let mut info_lines = Vec::new();

    for line in input.lines().flatten() {
        if let Some(formatted) = parse_log(&line, &rules) {
            let lower = line.to_lowercase();
            if lower.contains("panic") || lower.contains("oops") {
                critical_lines.push(formatted);
            } else if lower.contains("error") || lower.contains("fail") {
                error_lines.push(formatted);
            } else if lower.contains("warn") {
                warning_lines.push(formatted);
            } else {
                info_lines.push(formatted);
            }
        } else {
            info_lines.push(line); // unmatched line
        }
    }

    // Step 3: Show selection menu
    loop {
        display_menu(&critical_lines, &error_lines, &warning_lines, &info_lines);
    }
}

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
        format!("ℹ️  Ok ({})", info_lines.len()),
        "🚪 Exit".to_string(),
    ];

    let selection = Select::new("Section:", options.clone())
        .with_help_message("Use arrows ↑↓ and press Enter. Press Esc to quit.")
        .prompt();

    match selection {
        Ok(choice) => {
            let output = match choice.as_str() {
                c if c.starts_with("🔥") => critical_lines,
                c if c.starts_with("❌") => error_lines,
                c if c.starts_with("⚠️") => warning_lines,
                c if c.starts_with("ℹ️") => info_lines,
                c if c.starts_with("🚪") => {
                    println!("Exiting...");
                    std::process::exit(0);
                }
                _ => {
                    println!("Invalid selection.");
                    return;
                }
            };

            let content = output.join("\n");

            let mut pager = Command::new("less")
                .arg("-R")
                .stdin(Stdio::piped())
                .spawn()
                .expect("Failed to launch pager");

            if let Some(stdin) = pager.stdin.as_mut() {
                stdin
                    .write_all(content.as_bytes())
                    .expect("Failed to write to pager");
            }

            pager.wait().expect("Failed to wait on pager");
        }
        Err(_) => {
            println!("Exited via Esc or input error.");
            std::process::exit(0);
        }
    }
}
