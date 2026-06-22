//! `dotfiles-core` — the pure state model behind `dotfiles-tui` (ADR-004).
//!
//! No UI, no interactive I/O. This crate owns the derived dotfiles state that
//! both front-ends project (ADR-005). v0.1 scope: parse the TOML manifest
//! (ADR-003) into the self-documenting catalog (ADR-002). Deploy-status
//! derivation and the always-fresh projection land in the next slices.

use serde::{Deserialize, Serialize};

/// How an entry is deployed (ADR-001 #1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Symlink the target to the source in the repo (default).
    Symlink,
    /// Recursively copy — for directories like nested git repos.
    Copy,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Symlink
    }
}

/// One managed dotfile — a row in the self-documenting catalog (ADR-002/003).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// Stable handle, e.g. `"zsh"`.
    pub name: String,
    /// Source path, relative to the repo root.
    pub path: String,
    /// Deploy target, relative to `$HOME`.
    pub target: String,
    /// Whether the entry is currently managed.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Deployment mode.
    #[serde(default)]
    pub mode: Mode,
    /// The durable *why* docstring (ADR-002). Advisory; may be absent.
    #[serde(default)]
    pub why: Option<String>,
}

fn default_true() -> bool {
    true
}

/// The parsed manifest: a TOML array-of-tables of `[[entry]]` (ADR-003).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default, rename = "entry")]
    pub entries: Vec<Entry>,
}

impl Manifest {
    /// Parse a TOML manifest string.
    pub fn from_toml(src: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_array_of_tables_with_why() {
        let src = r#"
            [[entry]]
            name = "zsh"
            path = "zsh/.zshrc"
            target = ".zshrc"
            why = "Cross-machine shell baseline."

            [[entry]]
            name = "nvim"
            path = "nvim"
            target = ".config/nvim"
            mode = "symlink"
            enabled = false
        "#;
        let m = Manifest::from_toml(src).expect("parses");
        assert_eq!(m.entries.len(), 2);
        assert_eq!(m.entries[0].name, "zsh");
        assert_eq!(m.entries[0].mode, Mode::Symlink); // defaulted
        assert!(m.entries[0].enabled); // defaulted true
        assert_eq!(m.entries[0].why.as_deref(), Some("Cross-machine shell baseline."));
        assert!(!m.entries[1].enabled);
        assert!(m.entries[1].why.is_none());
    }
}
