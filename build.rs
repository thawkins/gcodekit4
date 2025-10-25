fn main() {
    slint_build::compile("src/ui.slint").unwrap();

    // Set build date/time as an environment variable
    let build_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
