pub mod bitmap;
pub mod glyph;
pub mod font;
pub mod script;
pub mod error;
pub mod parse;
pub mod bitmap_cache;
#[cfg(test)]
pub mod test;

pub use self::font::{BstFont,BstFontWeight};
pub use self::glyph::{BstGlyph,BstGlyphRaw,BstGlyphPos,BstGlyphGeo};
pub use self::error::{BstTextError,BstTextErrorSrc,BstTextErrorTy};
pub use self::script::{BstTextScript,BstTextLang};
pub use self::parse::parse_and_shape;
pub use self::bitmap::BstGlyphBitmap;
pub use self::bitmap_cache::BstGlyphBitmapCache;

use std::sync::Arc;
use crate::interface::bin::{Bin,BinStyle,PositionTy};
use crate::Basalt;

pub struct BasaltText {
	pub container: Arc<Bin>,
	pub bitmap_cache: BstGlyphBitmapCache,
	pub glyph_data: Vec<BstGlyphData>,
}

pub struct BstGlyphData {
	pub glyph: BstGlyph,
	pub bin: Arc<Bin>,
}

pub fn create_basalt_text<T: AsRef<str>>(basalt: &Arc<Basalt>, text: T, script: BstTextScript, lang: BstTextLang) -> Result<BasaltText, BstTextError> {
	let glyphs = parse_and_shape(text, script, lang)?;
	let mut bins = basalt.interface_ref().new_bins(glyphs.len() + 1);
	let container = bins.pop().unwrap();
	let height = glyphs.first().unwrap().glyph_raw.font.ascender - glyphs.first().unwrap().glyph_raw.font.descender;
	
	container.style_update(BinStyle {
		position_t: Some(PositionTy::FromParent),
		pos_from_t: Some(0.0),
		pos_from_l: Some(0.0),
		pos_from_r: Some(0.0),
		height: Some(height),
		overflow_y: Some(true),
		.. BinStyle::default()
	});
	
	let mut bitmap_cache = BstGlyphBitmapCache::new(basalt.clone());
	let mut glyph_data: Vec<BstGlyphData> = Vec::new();
	
	for glyph in glyphs {
		let bitmap = bitmap_cache.bitmap_for_glyph(&glyph)?;
		let bin = bins.pop().unwrap();
		container.add_child(bin.clone());
		
		bin.style_update(BinStyle {
			position_t: Some(PositionTy::FromParent),
			pos_from_l: Some((glyph.position.x + bitmap.bearing_x).floor()),
			pos_from_t: Some((glyph.position.y + bitmap.bearing_y).ceil()),
			width: Some(bitmap.width as f32),
			height: Some(bitmap.height as f32),
			back_image_atlas: Some(bitmap.coords.clone()),
			.. BinStyle::default()
		});
		
		glyph_data.push(BstGlyphData {
			glyph,
			bin
		})
	}
	
	Ok(BasaltText {
		container,
		bitmap_cache,
		glyph_data
	})
}
