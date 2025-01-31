use std::path::Path;

fn main() -> miette::Result<()> {
    // build_spout2()?;

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_dir = repo_root.join("libs");

    let include_path = std::path::PathBuf::from("include");

    // This assumes all your C++ bindings are in main.rs
    let mut b = autocxx_build::Builder::new("src/spout/mod.rs", [&include_path]).build()?;
    b.flag_if_supported("-std=c++17").compile("spout-library"); // arbitrary library name, pick anything

    println!("cargo:rerun-if-changed=src/spout/mod.rs");
    println!("cargo:rustc-link-lib=SpoutDX");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    Ok(())
}
