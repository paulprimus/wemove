use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=TAILWINDCSS");
    println!("cargo:rerun-if-changed=styles.css");
    println!("cargo:rerun-if-changed=src");

    let config = topcoat::tailwind::BuildConfig::new().input("styles.css");

    let config = match std::env::var_os("TAILWINDCSS") {
        Some(_) => config.executable_env("TAILWINDCSS"),
        None => config.executable(npm_shim()),
    };

    config
        .render()
        .expect("failed to build Tailwind stylesheet; run `npm install` in crates/web or set TAILWINDCSS to a Tailwind CLI executable");
}

fn npm_shim() -> PathBuf {
    let manifest_dir = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is set for build scripts"),
    );
    let shim = if cfg!(windows) {
        "tailwindcss.cmd"
    } else {
        "tailwindcss"
    };
    let shim = manifest_dir.join("node_modules").join(".bin").join(shim);
    if shim.exists() {
        println!("cargo:rerun-if-changed={}", shim.display());
    }
    shim
}
