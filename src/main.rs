//! `kotonoha` — CLI entry (argument parsing and UX). Domain logic comes from [`kotonoha_core`].

use std::io::{self, Read};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kotonoha")]
#[command(
    about = "Kotonoha / SLS developer CLI (see docs/cli-definition.md)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print CLI build identity and targeted specification bundle version.
    Version,
    /// RDE review output interchange (validate / emit skeleton).
    Rde {
        #[command(subcommand)]
        action: RdeAction,
    },
}

#[derive(Subcommand)]
enum RdeAction {
    /// Validate JSON against Phase 1 RDE interchange (`docs/rde-review-output.md` in kotonoha-spec).
    Validate {
        /// Fail if category items omit `summary` (spec SHOULD).
        #[arg(long)]
        strict: bool,
        /// JSON file, or `-` / omit for stdin.
        path: Option<PathBuf>,
    },
    /// Emit a minimal compliant JSON skeleton (stdout).
    Emit,
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Version => cmd_version(),
        Commands::Rde { action } => match action {
            RdeAction::Validate { strict, path } => cmd_rde_validate(strict, path.as_deref()),
            RdeAction::Emit => cmd_rde_emit(),
        },
    };
    process::exit(code);
}

fn cmd_version() -> i32 {
    println!("kotonoha {}", env!("CARGO_PKG_VERSION"));
    println!(
        "kotonoha-spec (target bundle): {}",
        kotonoha_core::TARGET_SPEC_BUNDLE
    );
    0
}

fn cmd_rde_emit() -> i32 {
    let skeleton = serde_json::json!({
        "rde_review_output": {
            "spec_version": kotonoha_core::TARGET_SPEC_BUNDLE,
            "subject_ref": "https://example.invalid/subject/REPLACE",
            "categories": {
                "preserved": [],
                "transformed": [],
                "complemented": [],
                "intentionally_unresolved": [],
                "lost": [],
                "deviation_risk": [],
                "next_update_policy": []
            }
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&skeleton).unwrap_or_else(|_| "{}".to_string())
    );
    0
}

fn cmd_rde_validate(strict: bool, path: Option<&std::path::Path>) -> i32 {
    let mut buf = Vec::new();
    match load_input(path, &mut buf) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    }
    let text = match String::from_utf8(buf) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("input is not valid UTF-8");
            return 1;
        }
    };
    match kotonoha_core::rde::validate_json(&text, strict) {
        Ok(warnings) => {
            for w in warnings {
                eprintln!("warning: {}", w);
            }
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            2
        }
    }
}

fn load_input(path: Option<&std::path::Path>, buf: &mut Vec<u8>) -> Result<(), String> {
    match path {
        None => io::stdin()
            .read_to_end(buf)
            .map(|_| ())
            .map_err(|e| format!("read stdin: {e}")),
        Some(p) if p.as_os_str() == "-" => io::stdin()
            .read_to_end(buf)
            .map(|_| ())
            .map_err(|e| format!("read stdin: {e}")),
        Some(p) => std::fs::read(p)
            .map(|b| buf.extend(b))
            .map_err(|e| format!("{}: {e}", p.display())),
    }
}
