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
		let gui_id = state.ui_elements.insert(
			row(&[
				column(&[
					rect().background_color(Color::RED),
					rect().background_color(Color::WHITE),
					rect().background_color(Color::GREEN),
					rect().background_color(Color::BLACK),
					rect().background_color(Color::WHITE),
					rect().background_color(Color::GREEN),
				]),
				rect().background_color(Color::BLACK),
				rect().background_color(Color::GREEN)
			]).margin(0.2)
		);

		let window = Window::new().title("Puppy Todo").ui(gui_id);
		state.windows.insert(window);
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Ok(pge::run(TodoApp::new()).await?)
}
