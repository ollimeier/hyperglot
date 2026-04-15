use std::collections::{HashMap, HashSet};

use crate::checks::get_all_checks;
use crate::enums::{LanguageStatus, LanguageValidity, OrthographyStatus, SupportLevel};
use crate::error::Result;
use crate::language::Language;
use crate::languages::Languages;
use crate::shaper::Shaper;

#[derive(Clone, Debug)]
pub struct CheckOptions {
    pub check: Vec<String>,
    pub validity: String,
    pub status: Vec<String>,
    pub orthography: Vec<String>,
    pub decomposed: bool,
    pub marks: bool,
    pub shaping: bool,
    pub shaping_threshold: f64,
}

impl Default for CheckOptions {
    fn default() -> Self {
        CheckOptions {
            check: vec![SupportLevel::Base.value().to_string()],
            validity: LanguageValidity::Draft.value().to_string(),
            status: vec![LanguageStatus::Living.value().to_string()],
            orthography: vec![OrthographyStatus::Primary.value().to_string()],
            decomposed: false,
            marks: false,
            shaping: false,
            shaping_threshold: 0.01,
        }
    }
}

pub struct CheckContext<'a> {
    pub characters: &'a HashSet<String>,
    pub options: &'a CheckOptions,
    pub shaper: Option<&'a Shaper>,
}

pub struct CharsetChecker {
    pub characters: HashSet<String>,
}

impl CharsetChecker {
    pub fn new(chars: impl IntoIterator<Item = String>) -> Self {
        let characters = chars
            .into_iter()
            .filter(|c| !c.trim().is_empty())
            .collect();
        CharsetChecker { characters }
    }

    pub fn get_supported_languages(
        &self,
        options: CheckOptions,
    ) -> Result<HashMap<String, HashMap<String, Language>>> {
        get_supported_languages_impl(&self.characters, None, &options)
    }

    pub fn supports_language(&self, iso: &str, options: &CheckOptions) -> Result<bool> {
        supports_language_impl(iso, &self.characters, None, options)
            .map(|s| !s.is_empty())
    }
}

pub struct FontChecker {
    pub characters: HashSet<String>,
    pub fontpath: String,
}

impl FontChecker {
    pub fn new(fontpath: &str) -> Result<Self> {
        let shaper = Shaper::new(fontpath)?;
        let chars: HashSet<String> = shaper
            .get_codepoints()
            .into_iter()
            .map(|c| c.to_string())
            .collect();
        Ok(FontChecker {
            characters: chars,
            fontpath: fontpath.to_string(),
        })
    }

    pub fn get_supported_languages(
        &self,
        mut options: CheckOptions,
    ) -> Result<HashMap<String, HashMap<String, Language>>> {
        options.shaping = true;
        let shaper = Shaper::new(&self.fontpath)?;
        get_supported_languages_impl(&self.characters, Some(&shaper), &options)
    }

    pub fn supports_language(&self, iso: &str, options: &CheckOptions) -> Result<bool> {
        let shaper = Shaper::new(&self.fontpath)?;
        supports_language_impl(iso, &self.characters, Some(&shaper), options)
            .map(|s| !s.is_empty())
    }
}

fn get_supported_languages_impl(
    characters: &HashSet<String>,
    shaper: Option<&Shaper>,
    options: &CheckOptions,
) -> Result<HashMap<String, HashMap<String, Language>>> {
    let languages = Languages::new(false, &options.validity, true)?;
    let mut support: HashMap<String, HashMap<String, Language>> = HashMap::new();
    let status_list = LanguageStatus::parse(&options.status);

    for (iso, lang) in languages.iter() {
        let lang_validity = lang.validity();
        if LanguageValidity::index(lang_validity) < LanguageValidity::index(&options.validity) {
            continue;
        }
        if !status_list.contains(&lang.status().to_string()) {
            continue;
        }

        let script_support = supports_language_impl(iso, characters, shaper, options)?;
        for (script, iso_list) in script_support {
            let entry = support.entry(script).or_default();
            for supported_iso in iso_list {
                if let Some(l) = languages.get(&supported_iso) {
                    entry.insert(supported_iso, l.clone());
                }
            }
        }
    }

    Ok(support)
}

fn supports_language_impl(
    iso: &str,
    characters: &HashSet<String>,
    shaper: Option<&Shaper>,
    options: &CheckOptions,
) -> Result<HashMap<String, Vec<String>>> {
    let language = Language::new(iso)?;
    let orth_include: Vec<&str> = options.orthography.iter().map(|s| s.as_str()).collect();
    let orthographies = language.get_check_orthographies(&orth_include);

    if orthographies.is_empty() {
        return Ok(HashMap::new());
    }

    let checks = get_all_checks();
    let mut support: HashMap<String, Vec<String>> = HashMap::new();

    for ort in &orthographies {
        let ctx = CheckContext {
            characters,
            options,
            shaper,
        };

        let mut supported = true;
        for check in &checks {
            // Skip shaping checks if no shaper
            if check.requires_font() && shaper.is_none() {
                continue;
            }

            // Check script condition
            if let Some(cond_script) = check.conditions_script() {
                if ort.script() != cond_script {
                    continue;
                }
            }

            // Check attribute conditions
            let cond_attrs = check.conditions_attributes();
            if !cond_attrs.is_empty() {
                let has_any = cond_attrs.iter().any(|a| {
                    ort.get(a)
                        .map(|v| !matches!(v, serde_yaml::Value::Null))
                        .unwrap_or(false)
                });
                if !has_any {
                    continue;
                }
            }

            match check.check(ort, &ctx) {
                Ok(true) => {}
                Ok(false) => {
                    supported = false;
                    break;
                }
                Err(e) => {
                    log::warn!("Check {} failed with error: {}", check.name(), e);
                    supported = false;
                    break;
                }
            }
        }

        if supported {
            support
                .entry(ort.script().to_string())
                .or_default()
                .push(iso.to_string());
        }
    }

    Ok(support)
}
