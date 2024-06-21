use crate::idgen::gen_id;
use crate::FontHandle;

pub struct Color {}

impl Color {
	pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
	pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
	pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
	pub const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
}

pub struct MouseArea {

}

impl MouseArea {
    pub fn on_clicked<F>(&self, mut f: F)
    where
        F: FnMut(),
    {
        f();
    }
}

pub fn mouse_area() -> MouseArea {
	MouseArea {}
}

#[derive(Clone, Debug)]
pub struct Window {
	pub id: usize,
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub body: GUIElement,
	pub lock_cursor: bool,
}

impl Window {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			title: "".to_string(),
			width: 800,
			height: 600,
			body: GUIElement::default(),
			lock_cursor: false,
		}
	}

	pub fn render(&self, el: GUIElement) {
		
	}
}

#[derive(Clone, Debug)]
pub enum Flex {
	Horizontal,
	Vertical,
	None
}

impl Default for Flex {
	fn default() -> Self {
		Flex::None
	}
}

#[derive(Clone, Debug, Default)]
pub struct GUIElement {
	pub grow: u32,
	pub children: Vec<GUIElement>,
	pub flex_dir: Flex,
	pub top_left_radius: f32,
	pub top_right_radius: f32,
	pub bottom_left_radius: f32,
	pub bottom_right_radius: f32,
	pub text: Option<String>,
	pub background_color: Option<[f32; 4]>,
	pub font_size: u32,
	pub camera_id: Option<usize>,
	pub font: Option<FontHandle>
}

impl GUIElement {
	pub fn add(mut self, child: GUIElement) -> Self {
		self.children.push(child);
		self
	}

	pub fn add_many(mut self, children: Vec<GUIElement>) -> Self {
		self.children.extend(children);
		self
	}

	pub fn background_color(mut self, color: [f32; 4]) -> Self {
		self.background_color = Some(color);
		self
	}

	pub fn grow(mut self, grow: u32) -> Self {
		self.grow = grow;
		self
	}

	pub fn camera(mut self, camera_id: usize) -> Self {
		self.camera_id = Some(camera_id);
		self
	}

	pub fn font(mut self, font: FontHandle) -> Self {
		self.font = Some(font);
		self
	}
}

pub fn vstack() -> GUIElement {
	GUIElement {
		flex_dir: Flex::Vertical,
		..Default::default()
	}
}

pub fn hstack() -> GUIElement {
	GUIElement {
		flex_dir: Flex::Horizontal,
		..Default::default()
	}
}

pub fn rect() -> GUIElement {
	GUIElement {
		background_color: Some(Color::WHITE),
		..Default::default()
	}
}

pub fn list() -> GUIElement {
	GUIElement {
		..Default::default()
	}
}

pub fn cameara(camera_id: usize) -> GUIElement {
	GUIElement {
		camera_id: Some(camera_id),
		..Default::default()
	}
}

pub fn text(t: &str) -> GUIElement {
	GUIElement {
		text: Some(t.to_string()),
		..Default::default()
	}
}