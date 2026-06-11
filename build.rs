// Rebuild (and re-embed framework assets) whenever anything under framework/
// changes — rust-embed embeds at compile time in release builds, so without
// this an edited CSS/HTML/JS asset can be silently missing from the binary.
fn main() {
    println!("cargo:rerun-if-changed=framework");
}
