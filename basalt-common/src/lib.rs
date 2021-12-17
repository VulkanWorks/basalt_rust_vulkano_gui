extern crate ilmenite;

pub use ilmenite::{ImtHoriAlign, ImtTextWrap, ImtVertAlign};

#[derive(Default, Debug)]
/// This struct should not be directly constructed. Use the `style!` macro instead.
/// Changes to this struct will not obey semver. This struct's fields should be considered private.
pub struct Style {
    // Position & Dimension
    pub position: Option<Position>,
    pub pos_from_t: Option<PxOrPct>,
    pub pos_from_b: Option<PxOrPct>,
    pub pos_from_l: Option<PxOrPct>,
    pub pos_from_r: Option<PxOrPct>,
    pub vert_offset: Option<PxOrPct>,
    pub hori_offset: Option<PxOrPct>,
    pub width: Option<PxOrPct>,
    pub height: Option<PxOrPct>,
    pub height_offset: Option<PxOrPct>,
    pub width_offset: Option<PxOrPct>,
    // Appearance
    pub hidden: Option<bool>,
    pub opacity_pct: Option<f32>,
    pub custom_verts: Vec<CustomVert>,
    // Non-Visual
    pub pass_input: Option<bool>,
    // Outer Positioning
    pub margin_t: Option<PxOrPct>,
    pub margin_b: Option<PxOrPct>,
    pub margin_l: Option<PxOrPct>,
    pub margin_r: Option<PxOrPct>,
    // Inner Positioning
    pub padding_t: Option<PxOrPct>,
    pub padding_b: Option<PxOrPct>,
    pub padding_l: Option<PxOrPct>,
    pub padding_r: Option<PxOrPct>,
    pub overflow_hori: Option<bool>,
    pub overflow_vert: Option<bool>,
    pub scroll_vert: Option<PxOrPct>,
    pub scroll_hori: Option<PxOrPct>,
    // Text
    pub text: Option<String>,
    pub text_secret: Option<bool>,
    pub text_color: Option<Color>,
    pub text_height: Option<PxOrPct>,
    pub text_wrap: Option<ImtTextWrap>,
    pub text_vert_align: Option<ImtVertAlign>,
    pub text_hori_align: Option<ImtHoriAlign>,
    pub line_spacing: Option<PxOrPct>,
    pub line_limit: Option<usize>,
    // Border
    pub border_size_t: Option<f32>,
    pub border_size_b: Option<f32>,
    pub border_size_l: Option<f32>,
    pub border_size_r: Option<f32>,
    pub border_color_t: Option<Color>,
    pub border_color_b: Option<Color>,
    pub border_color_l: Option<Color>,
    pub border_color_r: Option<Color>,
    pub border_radius_tl: Option<f32>,
    pub border_radius_tr: Option<f32>,
    pub border_radius_bl: Option<f32>,
    pub border_radius_br: Option<f32>,
}

#[derive(Debug)]
pub enum Position {
    Window,
    Parent,
    Floating,
}

#[derive(Debug)]
pub struct Color([f32; 4]);

impl Color {
    pub fn from_srgbaf32(from: [f32; 4]) -> Self {
        todo!()
    }

    pub fn from_lrgbaf32(from: [f32; 4]) -> Self {
        todo!()
    }

    pub fn from_srgba8(from: [u8; 4]) -> Self {
        todo!()
    }

    pub fn from_lrgba8(from: [u8; 4]) -> Self {
        todo!()
    }

    pub fn from_srgba16(from: [u16; 4]) -> Self {
        todo!()
    }

    pub fn from_lrgba16(from: [u16; 4]) -> Self {
        todo!()
    }

    pub fn from_srgba_hex<F: AsRef<str>>(from: F) -> Self {
        todo!()
    }

    pub fn from_lrgba_hex<F: AsRef<str>>(from: F) -> Self {
        todo!()
    }

    pub fn as_lrgba(&self) -> [f32; 4] {
        self.0
    }

    pub fn as_srgba(&self) -> [f32; 4] {
        todo!()
    }
}


#[derive(Debug)]
pub enum PxOrPct {
    Px(f32),
    Pct(f32),
}

impl PxOrPct {
    pub fn from_px(px: f32) -> Self {
        PxOrPct::Px(px)
    }

    pub fn from_pct(pct: f32) -> Self {
        PxOrPct::Pct(pct)
    }

    pub fn as_px(&self, base: f32) -> f32 {
        match self {
            Self::Px(px) => *px,
            Self::Pct(pct) => base * (pct / 100.0)
        }
    }
}

#[derive(Debug)]
pub struct CustomVert(());
