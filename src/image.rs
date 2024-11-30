#[allow(dead_code)]
pub struct Image {
	width: u32,
	height: u32,
	data: Vec<u32>,
}

impl Image {
	pub fn new(width: u32, height: u32, fill: Option<Color>) -> Self {
		let size = width as usize * height as usize;
		let data = match fill {
			None => vec![0; size],
			Some(color) => vec![color.into(); size],
		};

		Self {
			width,
			height,
			data,
		}
	}

	pub fn data(&self) -> &[u32] {
		&self.data
	}

	pub fn data_mut(&mut self) -> &mut [u32] {
		&mut self.data
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
		todo!()
	}
}

pub struct Color {
	pub r: u8,
	pub g: u8,
	pub b: u8,
}

impl Color {
	pub fn new(r: u8, g: u8, b: u8) -> Self {
		Color { r, g, b }
	}
}

impl Into<u32> for Color {
	fn into(self) -> u32 {
		((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
	}
}

impl Into<Color> for u32 {
	fn into(self) -> Color {
		let bytes = self.to_be_bytes();

		Color {
			r: bytes[1],
			g: bytes[2],
			b: bytes[3],
		}
	}
}
