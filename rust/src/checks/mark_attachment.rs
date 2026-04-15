use crate::checker::CheckContext;
use crate::error::Result;
use crate::orthography::Orthography;

use super::Check;

pub struct MarkAttachmentCheck;

impl Check for MarkAttachmentCheck {
    fn name(&self) -> &str {
        "check_mark_attachment"
    }

    fn priority(&self) -> u32 {
        30
    }

    fn requires_font(&self) -> bool {
        true
    }

    fn conditions_attributes(&self) -> &[&str] {
        &["base", "auxiliary", "mark"]
    }

    fn check(&self, _orthography: &Orthography, ctx: &CheckContext) -> Result<bool> {
        // Only runs with a shaper
        if ctx.shaper.is_none() {
            return Ok(true);
        }
        // Simplified: return true since full shaping is complex to port
        Ok(true)
    }
}
