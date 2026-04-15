pub mod arabic_joining;
pub mod brahmi_conjuncts;
pub mod brahmi_halfforms;
pub mod combination_marks;
pub mod coverage;
pub mod mark_attachment;

use crate::checker::CheckContext;
use crate::error::Result;
use crate::orthography::Orthography;

pub trait Check: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> u32 {
        999
    }
    fn requires_font(&self) -> bool {
        false
    }
    fn conditions_script(&self) -> Option<&str> {
        None
    }
    fn conditions_attributes(&self) -> &[&str] {
        &[]
    }
    fn check(&self, orthography: &Orthography, ctx: &CheckContext) -> Result<bool>;
}

pub fn get_all_checks() -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = vec![
        Box::new(coverage::CoverageCheck),
        Box::new(mark_attachment::MarkAttachmentCheck),
        Box::new(arabic_joining::ArabicJoiningCheck),
        Box::new(brahmi_conjuncts::BrahmiConjunctsCheck),
        Box::new(brahmi_halfforms::BrahmiHalfformsCheck),
        Box::new(combination_marks::CombinationMarksCheck),
    ];
    checks.sort_by_key(|c| c.priority());
    checks
}
