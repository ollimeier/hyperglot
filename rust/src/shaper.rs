use crate::error::{HyperglotError, Result};

pub struct Shaper {
    data: Vec<u8>,
}

impl Shaper {
    pub fn new(fontpath: &str) -> Result<Self> {
        let data = std::fs::read(fontpath)?;
        // Verify the font can be parsed
        let _ = rustybuzz::Face::from_slice(&data, 0)
            .ok_or_else(|| HyperglotError::Font(format!("Cannot parse font: {}", fontpath)))?;
        Ok(Shaper { data })
    }

    fn get_face(&self) -> Option<rustybuzz::Face<'_>> {
        rustybuzz::Face::from_slice(&self.data, 0)
    }

    pub fn shape(&self, text: &str) -> Vec<(rustybuzz::GlyphInfo, rustybuzz::GlyphPosition)> {
        let face = match self.get_face() {
            Some(f) => f,
            None => return vec![],
        };
        let mut buf = rustybuzz::UnicodeBuffer::new();
        buf.push_str(text);
        let output = rustybuzz::shape(&face, &[], buf);
        let infos = output.glyph_infos().to_vec();
        let positions = output.glyph_positions().to_vec();
        infos.into_iter().zip(positions).collect()
    }

    pub fn get_nominal_glyph(&self, cp: u32) -> Option<u32> {
        let c = char::from_u32(cp)?;
        let ttf = ttf_parser::Face::parse(&self.data, 0).ok()?;
        Some(ttf.glyph_index(c)?.0 as u32)
    }

    pub fn get_codepoints(&self) -> Vec<char> {
        let mut chars = Vec::new();
        if let Ok(ttf) = ttf_parser::Face::parse(&self.data, 0) {
            if let Some(cmap) = ttf.tables().cmap {
                for subtable in cmap.subtables {
                    if subtable.is_unicode() {
                        subtable.codepoints(|cp| {
                            if let Some(c) = char::from_u32(cp) {
                                chars.push(c);
                            }
                        });
                    }
                }
            }
        }
        chars
    }
}
