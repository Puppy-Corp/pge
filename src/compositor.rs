use lyon::math::point;
use lyon::math::Point;
use lyon::path::Path;
use lyon::tessellation::BuffersBuilder;
use lyon::tessellation::FillOptions;
use lyon::tessellation::FillTessellator;
use lyon::tessellation::FillVertex;
use lyon::tessellation::VertexBuffers;
use thunderdome::Index;

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
	pub background_color: [f32; 3]
}

impl DrawRect {
    pub fn generate_vertices_indices(&self) -> (Vec<[f32; 2]>, Vec<u16>) {
        let mut builder = Path::builder();

        // Move to the starting point
        builder.begin(point(self.top_left[0] + self.top_left_radius, self.top_left[1]));

        // Top line and top right corner
        builder.line_to(point(self.top_right[0] - self.top_right_radius, self.top_right[1]));
        builder.cubic_bezier_to(
            point(self.top_right[0] - self.top_right_radius, self.top_right[1] + self.top_right_radius),
            point(self.top_right_radius, self.top_right_radius),
            point(self.top_right[0], self.top_right[1] + self.top_right_radius)
        );

        // Right line and bottom right corner
        builder.line_to(point(self.bottom_right[0], self.bottom_right[1] - self.bottom_right_radius));
        builder.cubic_bezier_to(
            point(self.bottom_right[0] - self.bottom_right_radius, self.bottom_right[1] - self.bottom_right_radius),
            point(self.bottom_right_radius, self.bottom_right_radius),
            point(self.bottom_right[0] - self.bottom_right_radius, self.bottom_right[1])
        );

        // Bottom line and bottom left corner
        builder.line_to(point(self.bottom_left[0] + self.bottom_left_radius, self.bottom_left[1]));
        builder.cubic_bezier_to(
            point(self.bottom_left[0] + self.bottom_left_radius, self.bottom_left[1] - self.bottom_left_radius),
            point(self.bottom_left_radius, self.bottom_left_radius),
            point(self.bottom_left[0], self.bottom_left[1] - self.bottom_left_radius)
        );

        // Left line and top left corner
        builder.line_to(point(self.top_left[0], self.top_left[1] + self.top_left_radius));
        builder.cubic_bezier_to(
            point(self.top_left[0] + self.top_left_radius, self.top_left[1] + self.top_left_radius),
            point(self.top_left_radius, self.top_left_radius),
			point(self.top_left[0] + self.top_left_radius, self.top_left[1])
        );

        builder.close();

        // Build the path
        let path = builder.build();

        // Prepare a VertexBuffers to store the geometry
        let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

        {
			#[derive(Copy, Clone, Debug)]
			struct MyVertex { position: [f32; 2] }

            // Create the tessellator
            let mut tessellator = FillTessellator::new();

            // Use the tessellator to generate vertices and indices
            tessellator.tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                    Point::new(vertex.position().x, vertex.position().y)
                }),
            ).unwrap();
        }

        // Extract vertices and indices and convert them to Vec<[f32; 2]> and Vec<u16>
        let vertices = buffers.vertices.iter().map(|&p| [p.x, p.y]).collect();
        let indices = buffers.indices.clone(); // Indices are already in the required format

        (vertices, indices)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DrawText {
	pub text: String,
	pub font_size: f32,
	pub font_color: [f32; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct CamView {
	pub camera_id: Index,
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32
}

#[derive(Debug, Clone, PartialEq)]
enum DrawItem {
	Rect(DrawRect),
	Text(DrawText),
	CamView(CamView)
}

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
	pub items: Vec<DrawItem>,
}

impl Lineariser {
	pub fn new() -> Self {
		Self {
			items: Vec::new()
		}
	}

	fn inner_linearize(&mut self, item: &GUIElement, outline: Option<Outline>) {
		let mut outline = outline.unwrap_or(Outline {
			left_up: [-1.0, 1.0],
			right_up: [1.0, 1.0],
			left_down: [-1.0, -1.0],
			right_down: [1.0, -1.0]
		});

		outline.left_up[0] += item.left_margin;
		outline.right_up[0] -= item.right_margin;
		outline.left_down[0] += item.left_margin;
		outline.right_down[0] -= item.right_margin;
		outline.left_up[1] -= item.top_margin;
		outline.right_up[1] -= item.top_margin;
		outline.left_down[1] += item.bottom_margin;
		outline.right_down[1] += item.bottom_margin;

		if let Some(width) = item.width {
			if item.anchor_left == true && item.anchor_right == false {
				outline.right_up[0] = outline.left_up[0] + width;
				outline.right_down[0] = outline.left_down[0] + width;
			} else if item.anchor_left == false && item.anchor_right == true {
				outline.left_up[0] = outline.right_up[0] - width;
				outline.left_down[0] = outline.right_down[0] - width;
			}
		}

		if let Some(height) = item.height {
			if item.anchor_top == true && item.anchor_bottom == false {
				outline.left_down[1] = outline.left_up[1] - height;
				outline.right_down[1] = outline.right_up[1] - height;
			} else if item.anchor_top == false && item.anchor_bottom == true {
				outline.left_up[1] = outline.left_down[1] + height;
				outline.right_up[1] = outline.right_down[1] + height;
			}
		}

		if let Some(background_color) = item.background_color {
			self.items.push(DrawItem::Rect(DrawRect {
				top_left: outline.left_up,
				top_right: outline.right_up,
				bottom_left: outline.left_down,
				bottom_right: outline.right_down,
				background_color,
				..Default::default()
			}));
		}

		if let Some(text) = &item.text {
			let font_size = item.font_size as f32;
			let text = DrawText {
				text: text.clone(),
				font_size,
				font_color: item.font_color
			};
			self.items.push(DrawItem::Text(text));
		}

		if let Some(camera_id) = item.camera_id {
			self.items.push(DrawItem::CamView(CamView {
				camera_id,
				x: (outline.left_up[0] + 1.0) / 2.0,
				y: (outline.left_down[1] + 1.0) / 2.0,
				w: (outline.right_up[0] - outline.left_up[0]) / 2.0,
				h: (outline.left_up[1] - outline.left_down[1]) / 2.0
			}));
		}

		if item.children.len() > 0 {	
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
					let up_y = outline.left_up[1];
					let down_y = outline.left_down[1];
					let left_x = outline.left_up[0];
					let right_x = outline.right_up[0];
					for child in item.children.iter() {
						let left_up = [left_x, up_y];
						let left_down = [left_x, down_y];
						let right_up = [right_x, up_y];
						let right_down = [right_x, down_y];
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

	pub fn linearize(&mut self, item: &GUIElement) {
		self.inner_linearize(item, None);
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct UICompositor {
	lineariser: Lineariser,
	pub positions: Vec<[f32; 3]>,
	pub indices: Vec<u16>,
	pub colors: Vec<[f32; 3]>,
	pub views_3d: Vec<CamView>
}

impl UICompositor {
	pub fn new() -> Self {
		Self {
			lineariser: Lineariser::new(),
			positions: Vec::new(),
			indices: Vec::new(),
			colors: Vec::new(),
			views_3d: Vec::new()
		}
	}

	pub fn process(&mut self, item: &GUIElement) {
		self.positions.clear();
		self.indices.clear();
		self.colors.clear();

		self.lineariser.linearize(item);

		for draw in &self.lineariser.items {
			println!("draw item: {:?}", draw);

			match draw {
				DrawItem::Rect(rect) => {
					let (vertices, indices) = rect.generate_vertices_indices();
					println!("vertices: {:?}", vertices);
					println!("indices: {:?}", indices);
					let current_offset = self.positions.len() as u16;
					self.positions.extend(vertices.iter().map(|&p| [p[0], p[1], 0.0]));
					let adjusted_indices: Vec<u16> = indices.iter().map(|&i| i + current_offset).collect();
					// self.positions.push(self.positions.last().unwrap().clone());
					self.indices.extend(adjusted_indices);
					// self.indices.push(self.indices.last().unwrap().clone());
					self.colors.extend(std::iter::repeat(rect.background_color).take(vertices.len()));
					// self.colors.push(self.colors.last().unwrap().clone());
				},
				DrawItem::Text(text) => {},
				DrawItem::CamView(view) => {
					self.views_3d.push(view.clone());
				}
			}
		}
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
		linearizer.linearize(&rect);

		assert_eq!(linearizer.items.len(), 1);
		let r = DrawRect {
			top_left: [-1.0, 1.0],
			top_right: [1.0, 1.0],
			bottom_left: [-1.0, -1.0],
			bottom_right: [1.0, -1.0],
			background_color: Color::RED,
			..Default::default()
		};
		assert_eq!(linearizer.items[0], DrawItem::Rect(r));
	}

	#[test]
	fn test_vstack() {
		let vstack = column(&[
			rect().background_color(Color::RED),
			rect().background_color(Color::GREEN)
		]);
		let mut linearizer = Lineariser::new();
		linearizer.linearize(&vstack);

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
		let hstack = row(&[
			rect().background_color(Color::RED),
			rect().background_color(Color::GREEN)
		]);
		let mut linearizer = Lineariser::new();
		linearizer.linearize(&hstack);

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
		let vstack = column(&[
			rect().background_color(Color::RED).grow(3),
			rect().background_color(Color::GREEN).grow(1)
		]);
		let mut linearizer = Lineariser::new();
		linearizer.linearize(&vstack);

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
		let hstack = row(&[
			rect().background_color(Color::RED).grow(3),
			rect().background_color(Color::GREEN).grow(1)
		]);
		let mut linearizer = Lineariser::new();
		linearizer.linearize(&hstack);

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

	#[test]
	fn test_text_rendering() {
		let vstack = column(&[
			text("row 1"),
			text("row 2"),
			text("row 3")
		]);

		let mut linearizer = Lineariser::new();
		linearizer.linearize(&vstack);

		println!("{:?}", linearizer.items);
	}
}