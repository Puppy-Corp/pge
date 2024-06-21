pub use crate::gui::*;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DrawRect {
	pub top_left: [f32; 2],
	pub top_right: [f32; 2],
	pub bottom_left: [f32; 2],
	pub bottom_right: [f32; 2],
	pub top_left_radius: f32,
	pub top_right_radius: f32,
	pub bottom_left_radius: f32,
	pub bottom_right_radius: f32,
	pub background_color: [f32; 4]
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DrawText {
	pub text: String,
	pub font_size: f32,
	pub font_color: [f32; 4],
}

#[derive(Debug, Clone, PartialEq)]
enum DrawItem {
	Rect(DrawRect),
	Text(DrawText)
}

// pub struct Compositor {
// 	pub vertices: Vec<f32>,
// 	pub indices: Vec<u16>
// }

// impl Compositor {
// 	pub fn new() -> Self {
// 		Self {
// 			vertices: Vec::new(),
// 			indices: Vec::new()
// 		}
// 	}

// 	fn inner_render(&self, item: &GuiItem, size: OutlineSize) {
// 		match item {
// 			GuiItem::HStack(view) => {
// 				println!("HStack");
// 				let one_width = 1.0 / view.items.len() as f32;
// 				for item in view.items.iter() {
// 					self.inner_render(item, OutlineSize {
// 						width: Size::Exact(one_width),
// 						height: Size::Fill
// 					});
// 				}
// 			},
// 			GuiItem::VStack(view) => {
// 				println!("VStack");
// 				let one_height = 1.0 / view.items.len() as f32;
// 				for item in view.items.iter() {
// 					self.inner_render(item, OutlineSize {
// 						width: Size::Fill,
// 						height: Size::Exact(one_height)
// 					});
// 				}
// 			},
// 			GuiItem::SceneCam(cam) => {
// 				println!("SceneCam: {}", cam.camera_id);
// 			},
// 			GuiItem::Text(text) => {
// 				println!("Text: {}", text.text);
// 			},
// 			GuiItem::SceneCam(cam) => {
// 				println!("SceneCam: {}", cam.camera_id);
// 			},
// 			GuiItem::None => {},
// 			GuiItem::Rect(r) => {

// 			},
// 			GuiItem::List(list) => {
// 				for item in list.items.iter() {
// 					self.inner_render(item, OutlineSize {
// 						width: Size::Fill,
// 						height: Size::Content
// 					});
// 				}
// 			}
// 		}
// 	}

// 	pub fn render<T: Into<GuiItem>>(&self, item: T) {
// 		let item = item.into();
// 		self.inner_render(&item, OutlineSize {
// 			width: Size::Fill,
// 			height: Size::Fill
// 		});
// 	}
// }

pub enum Size {
	Fill,
	Exact(f32),
	Content
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
	pub left_up: [f32; 2],
	pub right_up: [f32; 2],
	pub left_down: [f32; 2],
	pub right_down: [f32; 2]
}

impl Outline {
	pub fn left_height(&self) -> f32 {
		self.left_up[1] - self.left_down[1]
	}

	pub fn right_height(&self) -> f32 {
		self.right_up[1] - self.right_down[1]
	}

	pub fn top_width(&self) -> f32 {
		self.right_up[0] - self.left_up[0]
	}

	pub fn bottom_width(&self) -> f32 {
		self.right_down[0] - self.left_down[0]
	}
}

pub struct Lineariser {
	pub items: Vec<DrawItem>
}

impl Lineariser {
	pub fn new() -> Self {
		Self {
			items: Vec::new()
		}
	}

	fn inner_linearize(&mut self, item: &GUIElement, size: Option<Outline>) {
		if let Some(background_color) = item.background_color {
			match &size {
				Some(size) => {
					self.items.push(DrawItem::Rect(DrawRect {
						top_left: size.left_up,
						top_right: size.right_up,
						bottom_left: size.left_down,
						bottom_right: size.right_down,
						background_color,
						..Default::default()
					}));
				},
				None => {
					self.items.push(DrawItem::Rect(DrawRect {
						top_left: [-1.0, 1.0],
						top_right: [1.0, 1.0],
						bottom_left: [-1.0, -1.0],
						bottom_right: [1.0, -1.0],
						background_color,
						..Default::default()
					}));
				}
			}
		}

		if item.children.len() > 0 {
			let outline = size.unwrap_or(Outline {
				left_up: [-1.0, 1.0],
				right_up: [1.0, 1.0],
				left_down: [-1.0, -1.0],
				right_down: [1.0, -1.0]
			});
			
			match item.flex_dir {
				Flex::Horizontal => {
					let total_grow: f32 = item.children.iter().map(|child| child.grow.max(1) as f32).sum();
					let top_width = outline.top_width();
					let bottom_width = outline.bottom_width();
					let mut up_x = outline.left_up[0];
					let mut down_x = outline.left_down[0];
					for child in item.children.iter() {
						let left_up = [up_x, outline.left_up[1]];
						let left_down = [down_x, outline.left_down[1]];
						let ratio = child.grow.max(1) as f32 / total_grow;
						let flexible_top_width = ratio * top_width;
						let flexible_bottom_width = ratio * bottom_width;
						up_x += flexible_top_width;
						down_x += flexible_bottom_width;
						let right_up = [up_x, outline.right_up[1]];
						let right_down = [down_x, outline.right_down[1]];			
						self.inner_linearize(child, Some(Outline {
							left_up,
							right_up,
							left_down,
							right_down
						}));
					}
				},
				Flex::Vertical => {
					let total_grow: f32 = item.children.iter().map(|child| child.grow.max(1) as f32).sum();
					let left_height = outline.left_height();
					let right_height = outline.right_height();
					let mut left_y = outline.left_up[1];
					let mut right_y = outline.right_up[1];
					for child in item.children.iter() {
						let ratio = child.grow.max(1) as f32 / total_grow;
						let flexible_left_height = ratio * left_height;
						let flexible_right_height = ratio * right_height;		
						let top_left_up = [outline.left_up[0], left_y];
						let top_right_up = [outline.right_up[0], right_y];
						left_y -= flexible_left_height;
						right_y -= flexible_right_height;
						let top_left_down = [outline.left_down[0], left_y];
						let top_right_down = [outline.right_down[0], right_y];			
						self.inner_linearize(child, Some(Outline {
							left_up: top_left_up,
							right_up: top_right_up,
							left_down: top_left_down,
							right_down: top_right_down
						}));
					}
				},
				Flex::None => {
					let mut up_y = outline.left_up[1];
					let mut down_y = outline.left_down[1];
					for child in item.children.iter() {
						let left_up = [outline.left_up[0], up_y];
						let left_down = [outline.left_down[0], down_y];
						up_y = down_y;
						down_y -= child.grow as f32;
						let right_up = [outline.right_up[0], up_y];
						let right_down = [outline.right_down[0], down_y];
						self.inner_linearize(child, Some(Outline {
							left_up,
							right_up,
							left_down,
							right_down
						}));
					}
				}
			}
		}
	}

	pub fn linearize(&mut self, item: GUIElement) {
		let item = item.into();
		self.inner_linearize(&item, None);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_rectangle() {
		let rect = rect()
			.background_color(Color::RED);
		let mut linearizer = Lineariser::new();
		linearizer.linearize(rect);

		assert_eq!(linearizer.items.len(), 1);
		let r = DrawRect {
			top_left: [-1.0, 1.0],
			top_right: [1.0, 1.0],
			bottom_left: [-1.0, -1.0],
			bottom_right: [1.0, -1.0],
			background_color: [1.0, 0.0, 0.0, 1.0],
			..Default::default()
		};
		assert_eq!(linearizer.items[0], DrawItem::Rect(r));
	}

	#[test]
	fn test_vstack() {
		let vstack = vstack()
			.add(rect().background_color(Color::RED))
			.add(rect().background_color(Color::GREEN));
		let mut linearizer = Lineariser::new();
		linearizer.linearize(vstack);

		assert_eq!(linearizer.items.len(), 2);
		let r1 = DrawRect {
			top_left: [-1.0, 1.0],
			top_right: [1.0, 1.0],
			bottom_left: [-1.0, 0.0],
			bottom_right: [1.0, 0.0],
			background_color: Color::RED,
			..Default::default()
		};
		let r2 = DrawRect {
			top_left: [-1.0, 0.0],
			top_right: [1.0, 0.0],
			bottom_left: [-1.0, -1.0],
			bottom_right: [1.0, -1.0],
			background_color: Color::GREEN,
			..Default::default()
		};
		assert_eq!(linearizer.items[0], DrawItem::Rect(r1));
		assert_eq!(linearizer.items[1], DrawItem::Rect(r2));
	}

	#[test]
	fn test_hstack() {
		let hstack = hstack()
			.add(rect().background_color(Color::RED))
			.add(rect().background_color(Color::GREEN));
		let mut linearizer = Lineariser::new();
		linearizer.linearize(hstack);

		assert_eq!(linearizer.items.len(), 2);
		let r1 = DrawRect {
			top_left: [-1.0, 1.0],
			top_right: [0.0, 1.0],
			bottom_left: [-1.0, -1.0],
			bottom_right: [0.0, -1.0],
			background_color: Color::RED,
			..Default::default()
		};
		let r2 = DrawRect {
			top_left: [0.0, 1.0],
			top_right: [1.0, 1.0],
			bottom_left: [0.0, -1.0],
			bottom_right: [1.0, -1.0],
			background_color: Color::GREEN,
			..Default::default()
		};
		assert_eq!(linearizer.items[0], DrawItem::Rect(r1));
		assert_eq!(linearizer.items[1], DrawItem::Rect(r2));
	}

	#[test]
	fn test_vstack_grow() {
		let vstack = vstack()
			.add(rect().background_color(Color::RED).grow(3))
			.add(rect().background_color(Color::GREEN).grow(1));
		let mut linearizer = Lineariser::new();
		linearizer.linearize(vstack);

		assert_eq!(linearizer.items.len(), 2);
		let r1 = DrawRect {
			top_left: [-1.0, 1.0],
			top_right: [1.0, 1.0],
			bottom_left: [-1.0, -0.5],
			bottom_right: [1.0, -0.5],
			background_color: Color::RED,
			..Default::default()
		};
		let r2 = DrawRect {
			top_left: [-1.0, -0.5],
			top_right: [1.0, -0.5],
			bottom_left: [-1.0, -1.0],
			bottom_right: [1.0, -1.0],
			background_color: Color::GREEN,
			..Default::default()
		};
		assert_eq!(linearizer.items[0], DrawItem::Rect(r1));
		assert_eq!(linearizer.items[1], DrawItem::Rect(r2));
	}

	#[test]
	fn test_hstack_grow() {
		let hstack = hstack()
			.add(rect().background_color(Color::RED).grow(3))
			.add(rect().background_color(Color::GREEN).grow(1));
		let mut linearizer = Lineariser::new();
		linearizer.linearize(hstack);

		assert_eq!(linearizer.items.len(), 2);
		let r1 = DrawRect {
			top_left: [-1.0, 1.0],
			top_right: [0.5, 1.0],
			bottom_left: [-1.0, -1.0],
			bottom_right: [0.5, -1.0],
			background_color: Color::RED,
			..Default::default()
		};
		let r2 = DrawRect {
			top_left: [0.5, 1.0],
			top_right: [1.0, 1.0],
			bottom_left: [0.5, -1.0],
			bottom_right: [1.0, -1.0],
			background_color: Color::GREEN,
			..Default::default()
		};
		assert_eq!(linearizer.items[0], DrawItem::Rect(r1));
		assert_eq!(linearizer.items[1], DrawItem::Rect(r2));
	}
}