//! Format-preserving manifest edits for the mutating verbs (`enable`/`disable`/
//! `add`/`remove`).
//!
//! Reads and rewrites the manifest through `toml_edit` so that comments, key
//! ordering, and every entry's `why`/`spec` survive a write untouched — only the
//! one field (or one `[[entry]]`) being changed moves.

use crate::Mode;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};

/// Parse a manifest into an editable document.
pub fn parse(src: &str) -> Result<DocumentMut, toml_edit::TomlError> {
    src.parse()
}

/// Fields for a new `[[entry]]`. Defaulted fields (`enabled = true`,
/// `mode = "symlink"`) are omitted from the written table to keep it clean.
#[derive(Debug, Clone)]
pub struct NewEntry<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub target: &'a str,
    pub mode: Mode,
    pub why: Option<&'a str>,
}

/// Borrow the `[[entry]]` array of tables, creating an empty one if absent.
fn entries_mut(doc: &mut DocumentMut) -> &mut ArrayOfTables {
    if doc.get("entry").and_then(Item::as_array_of_tables).is_none() {
        doc["entry"] = Item::ArrayOfTables(ArrayOfTables::new());
    }
    doc["entry"]
        .as_array_of_tables_mut()
        .expect("just ensured it is an array of tables")
}

fn index_of(aot: &ArrayOfTables, name: &str) -> Option<usize> {
    aot.iter()
        .position(|t| t.get("name").and_then(Item::as_str) == Some(name))
}

/// Set an entry's `enabled` flag. Returns `false` if no entry has that name.
pub fn set_enabled(doc: &mut DocumentMut, name: &str, enabled: bool) -> bool {
    let aot = entries_mut(doc);
    match index_of(aot, name) {
        Some(i) => {
            aot.get_mut(i).expect("index from position")["enabled"] = value(enabled);
            true
        }
        None => false,
    }
}

/// Remove an entry by name. Returns `false` if no entry has that name.
pub fn remove_entry(doc: &mut DocumentMut, name: &str) -> bool {
    let aot = entries_mut(doc);
    match index_of(aot, name) {
        Some(i) => {
            aot.remove(i);
            true
        }
        None => false,
    }
}

/// Append a new entry. Errors if one with the same name already exists.
pub fn add_entry(doc: &mut DocumentMut, e: NewEntry) -> Result<(), String> {
    let aot = entries_mut(doc);
    if index_of(aot, e.name).is_some() {
        return Err(format!("entry '{}' already exists in the manifest", e.name));
    }
    let mut t = Table::new();
    t["name"] = value(e.name);
    t["path"] = value(e.path);
    t["target"] = value(e.target);
    if e.mode == Mode::Copy {
        t["mode"] = value("copy");
    }
    if let Some(why) = e.why {
        t["why"] = value(why);
    }
    aot.push(t);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Manifest;

    const SRC: &str = r#"# a manifest
[[entry]]
name = "zsh"
path = "zsh/.zshrc"
target = ".zshrc"
why = "shell baseline"

[[entry]]
name = "tmux"
path = "tmux/.tmux.conf"
target = ".tmux.conf"
"#;

    #[test]
    fn disable_flips_only_the_one_flag_and_keeps_why() {
        let mut doc = parse(SRC).unwrap();
        assert!(set_enabled(&mut doc, "zsh", false));
        let out = doc.to_string();
        assert!(out.contains("enabled = false"));
        assert!(out.contains("why = \"shell baseline\""), "why preserved");
        assert!(out.contains("# a manifest"), "comment preserved");
        // tmux untouched.
        let m = Manifest::from_toml(&out).unwrap();
        assert!(!m.entries.iter().find(|e| e.name == "zsh").unwrap().enabled);
        assert!(m.entries.iter().find(|e| e.name == "tmux").unwrap().enabled);
    }

    #[test]
    fn add_appends_minimal_entry_and_rejects_duplicates() {
        let mut doc = parse(SRC).unwrap();
        add_entry(
            &mut doc,
            NewEntry { name: "nvim", path: "nvim", target: ".config/nvim", mode: Mode::Symlink, why: Some("editor") },
        )
        .unwrap();
        let out = doc.to_string();
        let m = Manifest::from_toml(&out).unwrap();
        let nvim = m.entries.iter().find(|e| e.name == "nvim").unwrap();
        assert_eq!(nvim.target, ".config/nvim");
        assert!(nvim.enabled, "defaulted true (key omitted)");
        assert!(!out.contains("mode = "), "symlink mode omitted");
        assert_eq!(nvim.why.as_deref(), Some("editor"));

        let dup = add_entry(
            &mut doc,
            NewEntry { name: "nvim", path: "x", target: "y", mode: Mode::Symlink, why: None },
        );
        assert!(dup.is_err());
    }

    #[test]
    fn remove_drops_the_entry() {
        let mut doc = parse(SRC).unwrap();
        assert!(remove_entry(&mut doc, "tmux"));
        assert!(!remove_entry(&mut doc, "tmux"), "second remove is a no-op");
        let m = Manifest::from_toml(&doc.to_string()).unwrap();
        assert_eq!(m.entries.len(), 1);
        assert_eq!(m.entries[0].name, "zsh");
    }
}
