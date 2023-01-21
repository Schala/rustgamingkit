use rgk_core::tag4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChunkTag {
	ColorMap = tag4!(b"CMAP"),
}
