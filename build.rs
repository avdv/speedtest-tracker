use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::Path;
use std::process::Command;

fn main() {
    // If any files in locales change we want to rebuild to update translations
    println!("cargo:rerun-if-changed=locales");

    // Rebuild CSS if source CSS changes
    println!("cargo:rerun-if-changed=resources/css/rust/app.css");

    // Watch template files individually for content changes
    // Note: This still triggers on timestamp changes, but we'll check content hash
    println!("cargo:rerun-if-changed=templates");

    // Allow skipping CSS build via environment variable
    println!("cargo:rerun-if-env-changed=SKIP_CSS_BUILD");

    // Build Tailwind CSS only if needed
    if std::env::var("SKIP_CSS_BUILD").is_ok() {
        println!("cargo:warning=Skipping CSS build (SKIP_CSS_BUILD is set)");
    } else {
        build_tailwind_css_if_needed();
    }
}

fn build_tailwind_css_if_needed() {
    let input = "resources/css/rust/app.css";
    let output = "public/css/rust-app.css";
    let hash_file = "target/.css-build-hash";

    // Check if input file exists
    if !Path::new(input).exists() {
        eprintln!("Warning: {} not found, skipping CSS build", input);
        return;
    }

    // Calculate hash of source CSS and relevant template files
    let current_hash = calculate_source_hash(input);

    // Check if we need to rebuild by comparing hashes
    let needs_rebuild = match fs::read_to_string(hash_file) {
        Ok(stored_hash) if stored_hash.trim() == current_hash => !Path::new(output).exists(),
        Ok(stored_hash) => {
            println!(
                "cargo:warning=Rebuilding CSS due to hash change (stored: {}, current: {})",
                stored_hash, current_hash
            );
            true
        }
        Err(_) => true,
    };

    if !needs_rebuild {
        return;
    }

    // Check if tailwindcss is available
    let tailwind_check = Command::new("tailwindcss").args(["--help"]).output();

    if tailwind_check.is_err() {
        eprintln!("Warning: tailwindcss not found, skipping CSS build");
        return;
    }

    println!("cargo:warning=Building Tailwind CSS...");

    // Run tailwindcss build
    let status = Command::new("tailwindcss")
        .args(["-i", input, "-o", output, "--minify"])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("cargo:warning=Tailwind CSS built successfully");
            // Save the hash to avoid unnecessary rebuilds
            let _ = fs::write(hash_file, current_hash);
        }
        Ok(status) => {
            eprintln!("Warning: tailwindcss exited with status: {}", status);
        }
        Err(e) => {
            eprintln!("Warning: Failed to run tailwindcss: {}", e);
        }
    }
}

fn calculate_source_hash(css_file: &str) -> String {
    let mut hasher = DefaultHasher::new();

    // Hash the CSS source file
    if let Ok(mut file) = fs::File::open(css_file) {
        let mut contents = Vec::new();
        if file.read_to_end(&mut contents).is_ok() {
            contents.hash(&mut hasher);
        }
    }

    // Hash all template files recursively (they contain the Tailwind classes)
    hash_templates_recursive("templates", &mut hasher);

    format!("{:x}", hasher.finish())
}

fn hash_templates_recursive(dir: &str, hasher: &mut DefaultHasher) {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.path());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                if let Some(path_str) = path.to_str() {
                    hash_templates_recursive(path_str, hasher);
                }
            } else if path.extension().map(|ext| ext == "html").unwrap_or(false) {
                if let Ok(mut file) = fs::File::open(&path) {
                    let mut contents = Vec::new();
                    if file.read_to_end(&mut contents).is_ok() {
                        contents.hash(hasher);
                    }
                }
            }
        }
    }
}
