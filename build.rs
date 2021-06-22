extern crate embed_resource;

// I've never used this feature of Rust, however I like
// the idea of using the language itself for build files.
// unfortunately package managers are heavy.
fn main() {
    embed_resource::compile("stupid.rc");
}
