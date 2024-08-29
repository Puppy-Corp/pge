use crate::ArenaId;
use crate::Camera;
use crate::FontHandle;

pub struct Color {}

impl Color {
	pub const BLACK: [f32; 3] = [0.0, 0.0, 0.0];
	pub const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
	pub const RED: [f32; 3] = [1.0, 0.0, 0.0,];
	pub const GREEN: [f32; 3] = [0.0, 1.0, 0.0];
	pub const BLUE: [f32; 3] = [0.0, 0.0, 1.0];
	pub const YELLOW: [f32; 3] = [1.0, 1.0, 0.0];
	pub const CYAN: [f32; 3] = [0.0, 1.0, 1.0];
	pub const MAGENTA: [f32; 3] = [1.0, 0.0, 1.0];
	pub const GRAY: [f32; 3] = [0.5, 0.5, 0.5];
	pub const LIGHT_GRAY: [f32; 3] = [0.75, 0.75, 0.75];
	pub const DARK_GRAY: [f32; 3] = [0.25, 0.25, 0.25];
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

#[derive(Clone, Debug, Default)]
pub struct Window {
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub ui: Option<ArenaId<UIElement>>,
	pub lock_cursor: bool,
}

impl Window {
	pub fn new() -> Self {
		Self {
			title: "".to_string(),
			width: 800,
			height: 600,
			ui: None,
			lock_cursor: false,
		}
	}

	pub fn title(mut self, title: &str) -> Self {
		self.title = title.to_string();
		self
	}

	pub fn ui(mut self, ui: ArenaId<UIElement>) -> Self {
		self.ui = Some(ui);
		self
	}

	pub fn lock_cursor(mut self, lock: bool) -> Self {
		self.lock_cursor = lock;
		self
	}
}

pub fn window() -> Window {
	Window::new()
}

#[derive(Clone, Debug)]
pub enum Layout {
	Horizontal,
	Vertical,
	Stack
}

impl Default for Layout {
	fn default() -> Self {
		Layout::Stack
	}
}

#[derive(Clone, Debug, Default)]
pub struct UIElement {
	pub grow: u32,
	pub children: Vec<UIElement>,
	pub layout: Layout,
	pub top_left_radius: f32,
	pub top_right_radius: f32,
	pub bottom_left_radius: f32,
	pub bottom_right_radius: f32,
	pub top_margin: f32,
	pub left_margin: f32,
	pub right_margin: f32,
	pub bottom_margin: f32,
	pub text: Option<String>,
	pub background_color: Option<[f32; 3]>,
	pub font_size: u32,
	pub font_color: [f32; 4],
	pub camera_id: Option<ArenaId<Camera>>,
	pub font: Option<FontHandle>,
	pub height: Option<f32>,
	pub width: Option<f32>,
	pub anchor_left: bool,
	pub anchor_right: bool,
	pub anchor_top: bool,
	pub anchor_bottom: bool,
}

impl UIElement {
	pub fn new() -> Self {
		Self {
			..Default::default()
		}
	}

	pub fn add(mut self, child: UIElement) -> Self {
		self.children.push(child);
		self
	}

	pub fn add_many(mut self, children: Vec<UIElement>) -> Self {
		self.children.extend(children);
		self
	}

	pub fn background_color(mut self, color: [f32; 3]) -> Self {
		self.background_color = Some(color);
		self
	}

	pub fn grow(mut self, grow: u32) -> Self {
		self.grow = grow;
		self
	}

	pub fn camera(mut self, camera_id: ArenaId<Camera>) -> Self {
		self.camera_id = Some(camera_id);
		self
	}

	pub fn font(mut self, font: FontHandle) -> Self {
		self.font = Some(font);
		self
	}

	pub fn margin(mut self, margin: f32) -> Self {
		self.top_margin = margin;
		self.left_margin = margin;
		self.right_margin = margin;
		self.bottom_margin = margin;
		self
	}

	pub fn height(mut self, height: f32) -> Self {
		self.height = Some(height);
		self
	}

	pub fn width(mut self, width: f32) -> Self {
		self.width = Some(width);
		self
	}

	pub fn anchor_left(mut self) -> Self {
		self.anchor_left = true;
		self
	}

	pub fn anchor_right(mut self) -> Self {
		self.anchor_right = true;
		self
	}

	pub fn anchor_top(mut self) -> Self {
		self.anchor_top = true;
		self
	}

	pub fn anchor_bottom(mut self) -> Self {
		self.anchor_bottom = true;
		self
	}
}

pub fn column(children: &[UIElement]) -> UIElement {
	UIElement {
		layout: Layout::Vertical,
		children: children.to_vec(),
		..Default::default()
	}
}

pub fn row(children: &[UIElement]) -> UIElement {
	UIElement {
		layout: Layout::Horizontal,
		children: children.to_vec(),
		..Default::default()
	}
}

pub fn stack(children: &[UIElement]) -> UIElement {
	UIElement {
		children: children.to_vec(),
		..Default::default()
	}
}

pub fn float(children: &[UIElement]) -> UIElement {
	UIElement {
		children: children.to_vec(),
		..Default::default()
	}
}

pub fn empty() -> UIElement {
	UIElement {
		..Default::default()
	}
}

pub fn rect() -> UIElement {
	UIElement {
		background_color: Some(Color::WHITE),
		..Default::default()
	}
}

pub fn list() -> UIElement {
	UIElement {
		..Default::default()
	}
}

pub fn camera_view(camera_id: ArenaId<Camera>) -> UIElement {
	UIElement {
		camera_id: Some(camera_id),
		..Default::default()
	}
}

pub fn text(t: &str) -> UIElement {
	UIElement {
		text: Some(t.to_string()),
		..Default::default()
	}
}