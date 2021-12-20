use crate::scale5to8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
	pub red: f32,
	pub green: f32,
	pub blue: f32,
	pub alpha: f32,
}

impl Color {
	pub fn from_rgba5551(color: u16) -> Color {
		Color {
			red: (scale5to8((color & 31) as u8) as f32) / 255.0,
			green: (scale5to8(((color >> 5) & 31) as u8) as f32) / 255.0,
			blue: (scale5to8(((color >> 10) & 31) as u8) as f32) / 255.0,
			alpha: (!scale5to8(((color >> 15) & 31) as u8) as f32) / 255.0,
		}
	}

	pub fn to_bgr8880(&self) -> u32 {
		((self.blue * 255.0) as u32) << 24 | ((self.green * 255.0) as u32) << 16 |
			((self.red * 255.0) as u32) << 8
	}

	pub fn to_rgb888(&self) -> u32 {
		((self.red * 255.0) as u32) << 16 | ((self.green * 255.0) as u32) << 8 |
			(self.blue * 255.0) as u32
	}

	pub fn to_rgba8888(&self) -> u32 {
		((self.red * 255.0) as u32) << 24 | ((self.green * 255.0) as u32) << 16 |
			((self.blue * 255.0) as u32) << 8 | (self.alpha * 255.0) as u32
	}

	/// Returns a textual hex representation in the form of `#rrggbb`
	pub fn hex_rgb(&self) -> String {
		let r = (self.red * 255.0) as u8;
		let g = (self.green * 255.0) as u8;
		let b = (self.blue * 255.0) as u8;

		format!("#{:x}{:x}{:x}", r, g, b)
	}
}

#[derive(Clone, Debug)]
pub struct Texture {
	pub palette: Vec<Color>,
	pub indices: Vec<usize>,
	pub width: usize,
	pub height: usize,
}

impl Texture {
	pub fn new(width: usize, height: usize) -> Texture {
		Texture {
			palette: vec![],
			indices: vec![],
			width: width,
			height: height,
		}
	}

	/// Returns the (X, Y) coordinates of every instance of a specified palette index
	pub fn find_indices(&self, index: usize) -> Vec<(usize, usize)> {
		let mut indices = vec![];

		for y in 0..self.height {
			for x in 0..self.width {
				if self.indices[(y * self.width) + x] == index {
					indices.push((x, y));
				}
			}
		}

		indices
	}

	/// Optimise the palette, removing duplicate entries, and adjusting indices accordingly.
	pub fn optimize(&mut self) {
		let mut opt_pal = vec![];

		let mut opt_idx = vec![0; self.width * self.height];

		for c1 in self.palette.iter() {
			let mut duplicate = None;

			// check for duplicates
			for c2 in opt_pal.iter() {
				if *c1 == *c2 {
					duplicate = Some(*c2);
					break;
				}
			}

			if let Some(c2) = duplicate {
				// get the existing color's palette index and assign it
				if let Some(i) = find_palette_index(&opt_pal, &c2) {
					for (x, y) in self.find_indices(i).iter() {
						opt_idx[(y * self.width) + x] = i;
					}
				}
			} else {
				// update the index as the new palette's current length
				let i = opt_pal.len();

				if let Some(j) = find_palette_index(&self.palette, c1) {
					for (x, y) in self.find_indices(j).iter() {
						opt_idx[(y * self.width) + x] = i;
					}
					opt_pal.push(*c1);
				}
			}
		}

		self.palette = opt_pal;
		self.indices = opt_idx;
	}

	/// Uses the palette and indices to build a pixel array
	pub fn pixels(&self) -> Vec<Color> {
		self.indices.iter().map(|i| self.palette[*i]).collect()
	}
}

/// Returns the index of the specified color, if present
pub fn find_palette_index<'a, 'b>(palette: &'a [Color], color: &'b Color) -> Option<usize> {
	for i in 0..palette.len() {
		if *color == palette[i] {
			return Some(i);
		}
	}

	None
}
