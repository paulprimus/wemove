fn main() {
    println!("cargo:rerun-if-env-changed=TAILWINDCSS");

    topcoat::tailwind::BuildConfig::new()
        .executable_env("TAILWINDCSS")
        .render()
        .expect("failed to build Tailwind stylesheet; set TAILWINDCSS to a local Tailwind CLI executable");
}
