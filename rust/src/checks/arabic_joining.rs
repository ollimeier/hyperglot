use crate::checker::CheckContext;
use crate::error::Result;
use crate::orthography::Orthography;

use super::Check;

pub struct ArabicJoiningCheck;

impl Check for ArabicJoiningCheck {
    fn name(&self) -> &str {
        "check_arabic_joining"
    }

    fn priority(&self) -> u32 {
        40
    }

    fn requires_font(&self) -> bool {
        true
    }

    fn conditions_script(&self) -> Option<&str> {
        Some("Arabic")
    }

    fn conditions_attributes(&self) -> &[&str] {
        &["base", "auxiliary"]
    }

    fn check(&self, _orthography: &Orthography, ctx: &CheckContext) -> Result<bool> {
        if ctx.shaper.is_none() {
            return Ok(true);
        }
        Ok(true)
    }
}
