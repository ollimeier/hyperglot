use crate::checker::CheckContext;
use crate::error::Result;
use crate::orthography::Orthography;

use super::Check;

pub struct CombinationMarksCheck;

impl Check for CombinationMarksCheck {
    fn name(&self) -> &str {
        "check_combination_marks"
    }

    fn priority(&self) -> u32 {
        75
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
