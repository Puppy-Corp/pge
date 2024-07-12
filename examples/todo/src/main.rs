use std::time::Duration;

use pge::*;
use text::FontMesh;
use tokio::time::sleep;
use ttf_parser::GlyphId;
use ttf_parser::OutlineBuilder;

struct TodoItem {
	text: String,
	completed: bool,
}

struct TodoApp {

}

impl TodoApp {
	pub fn new() -> Self {
		TodoApp {}
	}
}

impl pge::App for TodoApp {
	fn on_create(&mut self, state: &mut State) {
		let window = Window::new().title("TodoAPP")
			.ui(vstack(&[
				rect().background_color(Color::GREEN),
				rect().background_color(Color::RED)
			]));
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Ok(pge::run(TodoApp::new()).await?)
}
