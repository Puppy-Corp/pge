use crate::idgen::gen_id;
use crate::Camera;

pub struct Color {}

impl Color {
	pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
	pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
	pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
	pub const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
}

#[derive(Clone, Debug)]
pub struct SceneCam {
	pub camera_id: usize,
}

impl SceneCam {
	pub fn new(camera: &Camera) -> Self {
		Self {
			camera_id: camera.id
		}
	}
}

impl Into<GuiItem> for SceneCam {
	fn into(self) -> GuiItem {
		GuiItem::SceneCam(self)
	}
}

#[derive(Clone, Debug)]
pub struct Text {
	pub text: String,
	pub size: u32,
}

impl Text {
	pub fn size(mut self, size: u32) -> Self {
		self.size = size;
		self
	}

	pub fn text(mut self, text: &str) -> Self {
		self.text = text.to_string();
		self
	}
}

impl Into<GuiItem> for Text {
	fn into(self) -> GuiItem {
		GuiItem::Text(self)
	}
}

pub fn text(t: &str) -> Text {
	Text {
		text: t.to_string(),
		size: 12
	}
}

#[derive(Clone, Debug)]
pub struct HStack {
	pub items: Vec<GuiItem>
}

impl Into<GuiItem> for HStack {
	fn into(self) -> GuiItem {
		GuiItem::HStack(self)
	}
}

impl HStack {
	pub fn add<T: Into<GuiItem>>(mut self, item: T) -> Self {
		self.items.push(item.into());
		self
	}
}

#[derive(Clone, Debug)]
pub struct VStack {
	pub items: Vec<GuiItem>
}

impl Into<GuiItem> for VStack {
	fn into(self) -> GuiItem {
		GuiItem::VStack(self)
	}
}



impl VStack {
	pub fn add<T: Into<GuiItem>>(mut self, item: T) -> Self {
		self.items.push(item.into());
		self
	}
}

pub fn list() -> List {
	List {
		items: Vec::new()
	}
}

#[derive(Clone, Debug)]
pub struct List {
	pub items: Vec<GuiItem>
}

impl Into<GuiItem> for List {
	fn into(self) -> GuiItem {
		GuiItem::List(self)
	}
}

impl List {
	pub fn add<T: Into<GuiItem>>(mut self, item: T) -> Self {
		self.items.push(item.into());
		self
	}

	pub fn add_many<T: Into<GuiItem>>(mut self, items: Vec<T>) -> Self {
		for item in items {
			self.items.push(item.into());
		}
		self
	}	
}

#[derive(Clone, Debug)]
pub struct Rect {
	pub top_left_radius: f32,
	pub top_right_radius: f32,
	pub bottom_left_radius: f32,
	pub bottom_right_radius: f32,
	pub radius: f32,
	pub background_color: [f32; 4],
}

impl Rect {
	pub fn top_left_radius(mut self, radius: f32) -> Self {
		self.top_left_radius = radius;
		self
	}

	pub fn top_right_radius(mut self, radius: f32) -> Self {
		self.top_right_radius = radius;
		self
	}

	pub fn bottom_left_radius(mut self, radius: f32) -> Self {
		self.bottom_left_radius = radius;
		self
	}

	pub fn bottom_right_radius(mut self, radius: f32) -> Self {
		self.bottom_right_radius = radius;
		self
	}

	pub fn radius(mut self, radius: f32) -> Self {
		self.radius = radius;
		self
	}

	pub fn background_color(mut self, color: [f32; 4]) -> Self {
		self.background_color = color;
		self
	}
}

impl Into<GuiItem> for Rect {
	fn into(self) -> GuiItem {
		GuiItem::Rect(self)
	}
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
pub enum GuiItem {
	None,
	HStack(HStack),
	VStack(VStack),
	List(List),
	Text(Text),
	SceneCam(SceneCam),
	Rect(Rect)
}

#[derive(Clone, Debug)]
pub struct Window {
	pub id: usize,
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub body: GuiItem,
	pub lock_cursor: bool,
}

// impl Into<&GuiItem> for View {
// 	fn into(self) -> &GuiItem {
// 		&GuiItem::View(self)
// 	}
// }

impl Window {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			title: "".to_string(),
			width: 800,
			height: 600,
			body: GuiItem::None,
			lock_cursor: false,
		}
	}

	pub fn render<T: Into<GuiItem>>(&self, item: T) {
		
	}
}

pub struct Flow {
	pub items: Vec<GuiItem>
}

pub fn flow() -> Flow {
	Flow {
		items: Vec::new()
	}
}

#[derive(Clone, Debug)]
pub enum FlexDirection {
	Row,
	Column
}

impl Default for FlexDirection {
	fn default() -> Self {
		FlexDirection::Column
	}
}

#[derive(Clone, Debug, Default)]
pub struct GUIElement {
	pub grow: u32,
	pub children: Vec<GUIElement>,
	pub flex_dir: FlexDirection,
	pub top_left_radius: f32,
	pub top_right_radius: f32,
	pub bottom_left_radius: f32,
	pub bottom_right_radius: f32,
	pub text: Option<String>,
	pub background_color: Option<[f32; 4]>,
	pub font_size: u32,
}

impl GUIElement {
	pub fn add(mut self, child: GUIElement) -> Self {
		self.children.push(child);
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
}

pub fn vstack() -> GUIElement {
	GUIElement {
		..Default::default()
	}
}

pub fn hstack() -> GUIElement {
	GUIElement {
		flex_dir: FlexDirection::Row,
		..Default::default()
	}
}

pub fn rect() -> GUIElement {
	GUIElement {
		background_color: Some(Color::WHITE),
		..Default::default()
	}
}