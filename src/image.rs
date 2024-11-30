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

	pub fn rect(&mut self, x: u32, y: u32, width: u32, height: u32, clr: Color) {
		let x_start = x as usize;
		let x_end = (x + width) as usize;

		for idx in x_start..x_end {
			for idy in y as usize..y as usize + height as usize {
				let data_idx = idx + idy * self.width as usize;
				self.data[data_idx] = clr.into();
			}
		}
	}

	/// Primitive, naive line drawing funciton. lerps the points together and
	/// draws a rect of the specified width at that location.
	pub fn line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, width: u32, clr: Color) {
		let start_x = x1.min(x2) as f32;
		let start_y = y1.min(y2) as f32;
		let end_x = x1.max(x2) as f32;
		let end_y = y1.max(y2) as f32;

		tracing::trace!("start_x = {start_x} / end_x = {end_x}");
		tracing::trace!("start_y = {start_y} / end_y = {end_y}");

		let dx = end_x - start_x;
		let dy = end_y - start_y;
		let long = dx.max(dy);

		for idx in 0..long as usize {
			let x = start_x + dx * (idx as f32 / long);
			let y = start_y + dy * (idx as f32 / long);

			self.rect(x as u32, y as u32, width, width, clr);
		}
	}
}

#[derive(Copy, Clone, Debug)]
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
