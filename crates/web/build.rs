fn main() {
    println!("cargo:rerun-if-env-changed=TAILWINDCSS");
    println!("cargo:rerun-if-changed=styles.css");

    topcoat::tailwind::BuildConfig::new()
        .executable_env("TAILWINDCSS")
        .input("styles.css")
        .render()
        .expect("failed to build Tailwind stylesheet; set TAILWINDCSS to a local Tailwind CLI executable");
}
