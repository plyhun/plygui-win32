#[cfg(feature = "manifest")]
extern crate embed_resource;

fn main() {
    #[cfg(feature = "manifest")]
	embed_resource::compile("plygui.rc");
}