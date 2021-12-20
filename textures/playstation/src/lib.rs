pub mod tim;

use std::fs;

use rgk_core::texture::{
	Color,
	find_palette_index,
	Texture
};

use tim::*;

#[cfg(feature = "import")]
pub fn read_tim(filepath: &str) -> Result<Texture, TIMImportError> {
	let input = fs::read(filepath)?;
	let tex = PSXTexture::read(&mut input.as_slice())?;

	let mut texture = Texture::new((tex.img_header.width / 2) as usize, tex.img_header.height as usize);

	if tex.header.flags.contains(Flags::INDEXED) {
		if let Some(ref palette) = tex.palette {
			texture.palette = palette.iter().map(|c| Color::from_rgba5551(*c)).collect();
		}

		if let ImageData::Indexed(ref indices) = tex.data {
			texture.indices = indices.iter().map(|i| *i as usize).collect();
		}
	} else if tex.header.flags.contains(Flags::BPP_16) {
		if let ImageData::BPP16(ref colors) = tex.data {
			for color in colors.iter() {
				// Since these are direct color values, we'll have to build a palette.

				let c = Color::from_rgba5551(*color);

				if let Some(i) = find_palette_index(&texture.palette, &c) {
					texture.indices.push(i);
				} else {
					let i = texture.palette.len();
					texture.palette.push(c);
					texture.indices.push(i);
				}
			}
		}
	}

	texture.optimize();
	Ok(texture)
}
