use gifed::{block::Palette, videogif::Frame, writer::ImageBuilder, Gif};

use crate::Vec2;

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

	pub fn gif(&self) -> Gif {
		let mut palette = self.data.clone();
		palette.sort();
		palette.dedup();

		let indicies: Vec<u8> = self
			.data
			.clone()
			.into_iter()
			.map(|pix| palette.iter().position(|&x| x == pix).unwrap() as u8)
			.collect();

		let palette: Vec<gifed::Color> = palette
			.into_iter()
			.map(|c| {
				let b = c.to_be_bytes();
				gifed::Color::from([b[3], b[2], b[1]])
			})
			.collect();

		let img = ImageBuilder::new(self.width as u16, self.height as u16).build(indicies).unwrap();
		let mut gif = Gif::new(self.width as u16, self.height as u16);
		gif.set_palette(Some(Palette::try_from(palette).unwrap()));
		gif.push(img);

		gif
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn fill(&mut self, clr: Color) {
		self.data.fill(clr.into());
	}

	pub fn rect(&mut self, pos: Vec2<u32>, dim: Vec2<u32>, clr: Color) {
		let x_start = pos.x as usize;
		let x_end = (pos.x + dim.x) as usize;

		for idx in x_start..x_end {
			for idy in pos.y as usize..pos.y as usize + dim.y as usize {
				let data_idx = idx + idy * self.width as usize;
				self.data[data_idx] = clr.into();
			}
		}
	}

	/// Primitive, naive line drawing funciton. lerps the points together and
	/// draws a rect of the specified width at that location.
	pub fn line(&mut self, p1: Vec2<u32>, p2: Vec2<u32>, width: u32, clr: Color) {
		let start_x = p1.x.min(p2.x) as f32;
		let start_y = p1.y.min(p2.y) as f32;
		let end_x = p1.x.max(p2.x) as f32;
		let end_y = p1.y.max(p2.y) as f32;

		tracing::trace!("start_x = {start_x} / end_x = {end_x}");
		tracing::trace!("start_y = {start_y} / end_y = {end_y}");

		let dx = end_x - start_x;
		let dy = end_y - start_y;
		let long = dx.max(dy);

		let dim = Vec2::new(width, width);
		for idx in 0..long as usize {
			let x = start_x + dx * (idx as f32 / long);
			let y = start_y + dy * (idx as f32 / long);

			self.rect(Vec2::new(x, y).as_u32(), dim, clr);
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
