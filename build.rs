fn main() {
    // if any files in locales change we want to rebuild to update translations
    println!("cargo:rerun-if-changed=locales");
}
