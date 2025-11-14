use chrono::Utc;

fn main() {
    // Generate build timestamp
    let build_date = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Build Slint UI
    slint_build::compile("ui.slint").unwrap();
}
