fn main() {
    // Tauri embeds the frontend at compile time and needs the folder. A
    // plain `cargo build` or `cargo test` has no UI build, so a placeholder
    // page stands in. `npm run build` in `ui/` replaces it.
    let dist = std::path::Path::new("ui/dist");
    if !dist.join("index.html").exists() {
        std::fs::create_dir_all(dist).expect("create the placeholder UI folder");
        std::fs::write(
            dist.join("index.html"),
            "<!doctype html><title>Diffy</title><p>Run npm run build in apps/desktop/ui.</p>",
        )
        .expect("write the placeholder UI page");
    }
    println!("cargo:rerun-if-changed=ui/dist");
    tauri_build::build();
}
