use std::collections::HashMap;

use crate::enums::{LanguageValidity, OrthographyStatus};
use crate::error::Result;
use crate::language::Language;
use crate::loader::db_path;

pub struct Languages {
    pub langs: HashMap<String, Language>,
}

impl Languages {
    pub fn new(strict: bool, validity: &str, inherit: bool) -> Result<Self> {
        let path = db_path();
        let mut langs = HashMap::new();

        let entries = std::fs::read_dir(&path)?;
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            if fname_str.starts_with('.') {
                continue;
            }
            if !fname_str.ends_with(".yaml") && !fname_str.ends_with(".yml") {
                continue;
            }
            let iso = fname_str
                .trim_end_matches(".yaml")
                .trim_end_matches(".yml")
                .trim_end_matches('_')
                .to_string();

            match Language::new_with_options(&iso, None, inherit) {
                Ok(lang) => {
                    langs.insert(iso, lang);
                }
                Err(e) => {
                    log::warn!("Failed to load language '{}': {}", iso, e);
                }
            }
        }

        if !strict {
            lax_macrolanguages(&mut langs);
        }

        filter_by_validity(&mut langs, validity);
        set_defaults(&mut langs);

        Ok(Languages { langs })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Language)> {
        self.langs.iter()
    }

    pub fn get(&self, iso: &str) -> Option<&Language> {
        self.langs.get(iso)
    }
}

fn lax_macrolanguages(langs: &mut HashMap<String, Language>) {
    let mut to_remove: Vec<String> = Vec::new();

    let preferred_individual: Vec<(String, Vec<String>)> = langs
        .iter()
        .filter_map(|(iso, lang)| {
            if lang.get_raw("preferred_as_individual").is_some() {
                if !lang.orthographies.is_empty() {
                    if let Some(serde_yaml::Value::Sequence(includes)) =
                        lang.get_raw("includes")
                    {
                        let included: Vec<String> = includes
                            .iter()
                            .filter_map(|v| {
                                if let serde_yaml::Value::String(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        return Some((iso.clone(), included));
                    }
                }
            }
            None
        })
        .collect();

    for (_, includes) in preferred_individual {
        for included in includes {
            to_remove.push(included);
        }
    }

    for iso in to_remove {
        langs.remove(&iso);
    }
}

fn filter_by_validity(langs: &mut HashMap<String, Language>, validity: &str) {
    let allowed = LanguageValidity::index(validity);
    langs.retain(|_, lang| {
        let lang_validity = lang.validity();
        LanguageValidity::index(lang_validity) >= allowed
    });
}

fn set_defaults(langs: &mut HashMap<String, Language>) {
    for (_iso, lang) in langs.iter_mut() {
        if lang.orthographies.len() == 1 {
            let has_status = lang.orthographies[0].get("status").is_some()
                && !lang.orthographies[0].get_str("status").is_empty();
            if !has_status {
                lang.orthographies[0].data.insert(
                    serde_yaml::Value::String("status".to_string()),
                    serde_yaml::Value::String(
                        OrthographyStatus::Primary.value().to_string(),
                    ),
                );
            }
        }
    }
}
