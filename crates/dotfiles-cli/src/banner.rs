//! The `.DOTFILES` banner shown when `dotfiles` is run with no subcommand.
//!
//! Self-contained (no dependency on the agent-ways `agent-fmt` crate): the ANSI
//! Shadow figlet font is embedded at compile time, so the static release binary
//! carries everything it needs.

use figlet_rs::FIGlet;

const ANSI_SHADOW_FLF: &str = include_str!("../fonts/ansi-shadow.flf");

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const UNDERLINE: &str = "\x1b[4m";

/// Indigo → lavender gradient (distinct from ways' coral and attend's teal).
const GRADIENT_VIOLET: [&str; 7] = [
    "\x1b[38;5;63m",
    "\x1b[38;5;99m",
    "\x1b[38;5;135m",
    "\x1b[38;5;141m",
    "\x1b[38;5;177m",
    "\x1b[38;5;183m",
    "\x1b[38;5;189m",
];

/// Print the banner: a small over-title, the gradient figlet word, then a
/// subtitle and version line. Falls back to plain bold text if the font fails.
pub fn print() {
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subtitle = "self-documenting dotfiles · symlinks with a why";

    // Render to an owned String while the font is still alive (the FIGure
    // borrows the font, so we can't return it from the closure).
    let rendered: Option<String> = FIGlet::from_content(ANSI_SHADOW_FLF)
        .ok()
        .and_then(|font| font.convert(".DOTFILES").map(|fig| fig.to_string()));

    println!();
    println!("  {DIM}{UNDERLINE}D O T F I L E S{RESET}");
    println!();
    match rendered {
        Some(art) => {
            for (i, line) in art.lines().enumerate() {
                let color = GRADIENT_VIOLET[i % GRADIENT_VIOLET.len()];
                println!("{color}{line}{RESET}");
            }
        }
        None => println!("  {BOLD}.DOTFILES{RESET}"),
    }
    println!("  {DIM}{subtitle}{RESET}");
    println!("  {DIM}{version}{RESET}");
    println!();
}
