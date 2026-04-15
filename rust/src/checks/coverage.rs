use std::collections::HashSet;

use crate::checker::CheckContext;
use crate::error::Result;
use crate::orthography::Orthography;
use crate::parse::parse_chars;

use super::Check;

pub struct CoverageCheck;

impl Check for CoverageCheck {
    fn name(&self) -> &str {
        "check_coverage"
    }

    fn priority(&self) -> u32 {
        10
    }

    fn requires_font(&self) -> bool {
        false
    }

    fn conditions_attributes(&self) -> &[&str] {
        &["base", "auxiliary", "numerals", "punctuation", "currency"]
    }

    fn check(&self, orthography: &Orthography, ctx: &CheckContext) -> Result<bool> {
        let options = ctx.options;
        let characters = ctx.characters;
        let check_levels = &options.check;

        let mut support = true;

        for attr in check_levels {
            match attr.as_str() {
                "punctuation" | "numerals" | "currency" => {
                    let chars: HashSet<String> = match attr.as_str() {
                        "punctuation" => orthography.punctuation().into_iter().collect(),
                        "numerals" => orthography.numerals().into_iter().collect(),
                        "currency" => orthography.currency().into_iter().collect(),
                        _ => HashSet::new(),
                    };
                    if !chars.is_empty() && !chars.is_subset(characters) {
                        support = false;
                    }
                }
                "base" => {
                    let chars = orthography.get_chars("base", options.marks);
                    if chars.is_empty() {
                        support = false;
                        continue;
                    }
                    if !options.decomposed {
                        if !chars.is_subset(characters) {
                            support = false;
                        }
                    } else {
                        for c in &chars {
                            let decomposed: HashSet<String> =
                                parse_chars(c, true, false).into_iter().collect();
                            if !decomposed.is_subset(characters) {
                                support = false;
                                break;
                            }
                        }
                    }
                }
                "auxiliary" => {
                    let req_marks_aux = if options.marks {
                        orthography.auxiliary_marks()
                    } else {
                        orthography.required_auxiliary_marks()
                    };
                    let chars: HashSet<String> = orthography
                        .auxiliary_chars()
                        .into_iter()
                        .chain(req_marks_aux)
                        .collect();
                    if !chars.is_subset(characters) {
                        support = false;
                    }
                }
                _ => {}
            }
        }

        Ok(support)
    }
}
