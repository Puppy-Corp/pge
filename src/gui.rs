use crate::idgen::gen_id;
use crate::Camera;

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
}

#[derive(Clone, Debug)]
pub struct View {
		
}

impl View {
	// pub fn add(mut self, item: GuiItem) -> Self {
	// 	self
	// }

	pub fn add<T: Into<GuiItem>>(mut self, item: T) -> Self {
		self
	}
}

#[derive(Clone, Debug)]
pub enum GuiItem {
	None,
	View(View),
	Text(Text),
	SceneCam(SceneCam)
}

impl From<&mut View> for GuiItem {
	fn from(view: &mut View) -> Self {
		GuiItem::View(view.clone())
	}
}

#[derive(Clone, Debug)]
pub struct Window {
	pub id: usize,
	pub title: String,
	pub width: u32,
	pub height: u32,
	pub body: GuiItem
}

impl Into<GuiItem> for View {
	fn into(self) -> GuiItem {
		GuiItem::View(self)
	}
}



impl Window {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			title: "".to_string(),
			width: 800,
			height: 600,
			body: GuiItem::None
		}
	}
}

pub fn view() -> View {
	View {}
}