//! The Rust domain must not depend on Tauri, the UI, persistence or a provider.
use std::{fs, path::Path};

const DOMAIN: &[&str] = &[
    "diffy-core",
    "diffy-comparison",
    "diffy-git",
    "diffy-review",
];
const FORBIDDEN: &[&str] = &[
    "tauri",
    "diffy-storage",
    "diffy-providers",
    "diffy-desktop",
    "rusqlite",
    "sqlx",
    "reqwest",
];

#[test]
fn domain_crates_have_no_forbidden_dependencies() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in DOMAIN {
        let text = fs::read_to_string(root.join(name).join("Cargo.toml")).unwrap();
        let deps = text.split("[dependencies]").nth(1).unwrap_or("");
        for bad in FORBIDDEN {
            assert!(
                !deps.lines().any(|l| l.trim_start().starts_with(bad)),
                "{name} depends on {bad}"
            );
        }
    }
}
