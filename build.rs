use std::{fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=README.md");
    let readme = fs::read_to_string("README.md").unwrap();
    let output = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("README-lib.md");
    fs::write(output, prepare(&readme)).unwrap();
}

fn prepare(readme: &String) -> String {
    // This is a naive implementation to get things off the ground.
    // We just do a few things for this at the moment:
    // 1. Strip header stuff
    // 2. Replace the build document link
    // 3. Replace serde examples with ignore flags (to avoid feature flagging configuration in docs)
    let mut cleaned = String::new();
    let mut body = false;
    let mut feature_section = false;
    for line in readme.lines() {
        if !body {
            if line.starts_with("[docs]") {
                body = true;
            }
            continue;
        }

        // Add the line as is, unless it contains "(BUILD.md)"
        if line.contains("(BUILD.md)") {
            cleaned.push_str(&line.replace(
                "(BUILD.md)",
                "(https://github.com/paupino/rust-decimal/blob/master/BUILD.md)",
            ));
        } else if feature_section && line.starts_with("```rust") {
            cleaned.push_str("```ignore");
        } else {
            if !feature_section && line.starts_with("## Features") {
                feature_section = true;
            }
            cleaned.push_str(line);
        }
        cleaned.push('\n');
    }
    cleaned
}
