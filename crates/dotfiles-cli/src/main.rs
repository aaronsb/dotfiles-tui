//! `dotfiles-cli` — the non-interactive JSON surface (ADR-001 #4, ADR-004).
//!
//! The agent-facing front-end: fully scriptable, structured output. v0.1:
//! `status --manifest <file> --format json` prints the parsed catalog.

use clap::{Parser, Subcommand, ValueEnum};
use dotfiles_core::Manifest;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dotfiles-cli", version, about = "Agent-facing surface for dotfiles-tui")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the managed-dotfiles catalog.
    Status {
        /// Path to the TOML manifest.
        #[arg(long, default_value = ".dotfiles-manifest.toml")]
        manifest: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Json)]
        format: Format,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Json,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Status { manifest, format } => {
            let src = std::fs::read_to_string(&manifest)
                .map_err(|e| anyhow::anyhow!("reading {}: {e}", manifest.display()))?;
            let m = Manifest::from_toml(&src)?;
            match format {
                Format::Json => println!("{}", serde_json::to_string_pretty(&m)?),
            }
        }
    }
    Ok(())
}
