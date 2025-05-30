// src/main.rs
mod formatter;
mod parser;
mod rules;

use clap::Parser;
use inquire::Select;
use parser::parse_log;
use rules::load_rules;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "dmesg-analyzer")]
#[command(about = "Highlight and summarize dmesg logs with colors and rules", long_about = None)]
struct Cli {
    /// Path to the dmesg log file (use "-" for stdin)
    #[arg(short, long)]
    file: String,

    /// Path to the rule file (TOML format)
    #[arg(short, long, default_value = "rules/default_rules.toml")]
    rules: String,
}

fn main() {
    let cli = Cli::parse();
    let rules_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), cli.rules);
    eprintln!("Using rules from: {}", rules_path);

    let rules = load_rules(&rules_path);

    let input: Box<dyn BufRead> = if cli.file == "-" {
        Box::new(io::BufReader::new(io::stdin()))
    } else {
        let file = std::fs::File::open(&cli.file).expect("Failed to open log file");
        Box::new(io::BufReader::new(file))
    };

    let mut critical_lines = Vec::new();
    let mut error_lines = Vec::new();
    let mut warning_lines = Vec::new();
    let mut info_lines = Vec::new();

    for line in input.lines() {
        if let Ok(text) = line {
            if let Some(formatted) = parse_log(&text, &rules) {
                let lower = text.to_lowercase();
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
                info_lines.push(text); // Unmatched, assume info
            }
        }
    }

    loop {
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
                    c if c.starts_with("🔥") => &critical_lines,
                    c if c.starts_with("❌") => &error_lines,
                    c if c.starts_with("⚠️") => &warning_lines,
                    c if c.starts_with("ℹ️") => &info_lines,
                    c if c.starts_with("🚪") => {
                        println!("Exiting...");
                        break;
                    }
                    _ => {
                        println!("Invalid selection.");
                        continue;
                    }
                };

                let content = output.join("\n");

                // Spawn 'less' as a subprocess
                let mut pager = Command::new("less")
                    .arg("-R") // supports ANSI color
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
                break;
            }
        }
    }
}
