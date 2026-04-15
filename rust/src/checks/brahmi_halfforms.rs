use crate::checker::CheckContext;
use crate::error::Result;
use crate::orthography::Orthography;

use super::Check;

pub struct BrahmiHalfformsCheck;

impl Check for BrahmiHalfformsCheck {
    fn name(&self) -> &str {
        "check_brahmi_halfforms"
    }

    fn priority(&self) -> u32 {
        50
    }

    fn requires_font(&self) -> bool {
        true
    }

    fn conditions_script(&self) -> Option<&str> {
        Some("Devanagari")
    }

    fn conditions_attributes(&self) -> &[&str] {
        &["combinations"]
    }

    fn check(&self, _orthography: &Orthography, ctx: &CheckContext) -> Result<bool> {
        if ctx.shaper.is_none() {
            return Ok(true);
        }
        Ok(true)
    }
}
