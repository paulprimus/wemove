fn main() {
    topcoat::tailwind::BuildConfig::new()
        .render()
        .expect("failed to build Tailwind stylesheet");
}
