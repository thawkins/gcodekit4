fn main() {
    // Set include path for Slint compiler (used by both compile_with_config and inline slint! macros)
    let include_path = std::path::PathBuf::from("crates/gcodekit4-ui");
    println!(
        "cargo:rustc-env=SLINT_INCLUDE_PATH={}",
        include_path.display()
    );

    // Build Slint UI with include paths to crates/gcodekit4-ui
    let config = slint_build::CompilerConfiguration::new().with_include_paths(vec![include_path]);
    slint_build::compile_with_config("crates/gcodekit4-ui/ui.slint", config).unwrap();

    // Set build date/time as an environment variable
    let build_date = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
