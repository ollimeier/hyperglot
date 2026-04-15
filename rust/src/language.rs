use std::collections::HashMap;

use serde_yaml::{Mapping, Value};

use crate::enums::{CHARACTER_ATTRIBUTES, LanguageStatus, LanguageValidity, OrthographyStatus};
use crate::error::{HyperglotError, Result};
use crate::loader::load_language_data;
use crate::orthography::Orthography;

#[derive(Debug, Clone)]
pub struct Language {
    pub iso: String,
    pub data: Mapping,
    pub orthographies: Vec<Orthography>,
}

impl Language {
    pub fn new(iso: &str) -> Result<Self> {
        Self::new_with_options(iso, None, true)
    }

    pub fn new_no_inherit(iso: &str) -> Result<Self> {
        Self::new_with_options(iso, None, false)
    }

    pub fn new_with_options(iso: &str, data: Option<Mapping>, inherit: bool) -> Result<Self> {
        let data = if let Some(d) = data {
            d
        } else {
            let val = load_language_data(iso)?;
            match val {
                Value::Mapping(m) => m,
                _ => return Err(HyperglotError::LanguageNotFound(iso.to_string())),
            }
        };

        let mut lang = Language {
            iso: iso.to_string(),
            data: data.clone(),
            orthographies: vec![],
        };

        if inherit {
            lang.apply_defaults();
            lang.inherit_from_macrolanguage();
            lang.expand_orthographies()?;
        } else {
            // Still build orthographies without inheritance resolution
            lang.expand_orthographies_no_inherit()?;
        }

        Ok(lang)
    }

    fn apply_defaults(&mut self) {
        let defaults: &[(&str, Value)] = &[
            ("name", Value::Null),
            ("autonym", Value::Null),
            ("speakers", Value::Null),
            ("validity", Value::Null),
            ("status", Value::Null),
        ];
        for (k, v) in defaults {
            let key = Value::String(k.to_string());
            if !self.data.contains_key(&key) {
                self.data.insert(key, v.clone());
            }
        }
    }

    fn inherit_from_macrolanguage(&mut self) {
        if self.data.contains_key(Value::String("orthographies".to_string())) {
            return;
        }
        // Try to find macrolanguages that include this iso
        let db_path = crate::loader::db_path();
        if let Ok(entries) = std::fs::read_dir(&db_path) {
            for entry in entries.flatten() {
                let fname = entry.file_name();
                let fname_str = fname.to_string_lossy();
                if !fname_str.ends_with(".yaml") && !fname_str.ends_with(".yml") {
                    continue;
                }
                let macro_iso = fname_str
                    .trim_end_matches(".yaml")
                    .trim_end_matches(".yml")
                    .trim_end_matches('_')
                    .to_string();
                if macro_iso == self.iso {
                    continue;
                }
                if let Ok(val) = load_language_data(&macro_iso) {
                    if let Value::Mapping(m) = val {
                        let includes_key = Value::String("includes".to_string());
                        let orths_key = Value::String("orthographies".to_string());
                        if let Some(Value::Sequence(includes)) = m.get(&includes_key) {
                            let iso_val = Value::String(self.iso.clone());
                            if includes.contains(&iso_val) {
                                if let Some(orths) = m.get(&orths_key) {
                                    self.data.insert(orths_key, orths.clone());
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn expand_orthographies(&mut self) -> Result<()> {
        let raw_orths = match self.data.get(Value::String("orthographies".to_string())) {
            Some(Value::Sequence(s)) => s.clone(),
            _ => return Ok(()),
        };

        let _iso = self.iso.clone();
        let loader = move |iso_inner: &str, script: Option<&str>, status: Option<&str>| {
            let lang = Language::new_no_inherit(iso_inner)?;
            lang.get_orthography_owned(script, status)
        };

        let mut orths = Vec::new();
        for raw in raw_orths {
            if let Value::Mapping(m) = raw {
                let ort = Orthography::new(m, true, Some(&loader))?;
                orths.push(ort);
            }
        }
        self.orthographies = orths;
        Ok(())
    }

    fn expand_orthographies_no_inherit(&mut self) -> Result<()> {
        let raw_orths = match self.data.get(Value::String("orthographies".to_string())) {
            Some(Value::Sequence(s)) => s.clone(),
            _ => return Ok(()),
        };
        let mut orths = Vec::new();
        for raw in raw_orths {
            if let Value::Mapping(m) = raw {
                let ort = Orthography::new(m, false, None)?;
                orths.push(ort);
            }
        }
        self.orthographies = orths;
        Ok(())
    }

    pub fn get_raw(&self, key: &str) -> Option<&Value> {
        self.data.get(Value::String(key.to_string()))
    }

    pub fn name(&self) -> &str {
        match self.get_raw("name") {
            Some(Value::String(s)) => s.as_str(),
            _ => "",
        }
    }

    pub fn speakers(&self) -> u64 {
        match self.get_raw("speakers") {
            Some(Value::Number(n)) => n.as_u64().unwrap_or(0),
            Some(Value::String(s)) => s.parse().unwrap_or(0),
            _ => 0,
        }
    }

    pub fn validity(&self) -> &str {
        match self.get_raw("validity") {
            Some(Value::String(s)) if !s.is_empty() => s.as_str(),
            _ => LanguageValidity::Todo.value(),
        }
    }

    pub fn status(&self) -> &str {
        match self.get_raw("status") {
            Some(Value::String(s)) if !s.is_empty() => s.as_str(),
            _ => LanguageStatus::Living.value(),
        }
    }

    pub fn get_orthography_owned(&self, script: Option<&str>, status: Option<&str>) -> Result<Orthography> {
        let matches: Vec<&Orthography> = self
            .orthographies
            .iter()
            .filter(|o| {
                if let Some(s) = script {
                    if o.script() != s {
                        return false;
                    }
                }
                if let Some(st) = status {
                    if o.status() != st {
                        return false;
                    }
                }
                true
            })
            .collect();

        if matches.is_empty() {
            return Err(HyperglotError::OrthographyNotFound {
                script: script.unwrap_or("").to_string(),
                status: status.unwrap_or("").to_string(),
                lang: self.iso.clone(),
            });
        }

        let mut sorted = matches;
        sorted.sort_by_key(|o| OrthographyStatus::index(o.status()));
        Ok(sorted[0].clone())
    }

    pub fn get_orthography(&self, script: Option<&str>, status: Option<&str>) -> Result<&Orthography> {
        let idx = self
            .orthographies
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                if let Some(s) = script {
                    if o.script() != s {
                        return false;
                    }
                }
                if let Some(st) = status {
                    if o.status() != st {
                        return false;
                    }
                }
                true
            })
            .min_by_key(|(_, o)| OrthographyStatus::index(o.status()))
            .map(|(i, _)| i);

        match idx {
            Some(i) => Ok(&self.orthographies[i]),
            None => Err(HyperglotError::OrthographyNotFound {
                script: script.unwrap_or("").to_string(),
                status: status.unwrap_or("").to_string(),
                lang: self.iso.clone(),
            }),
        }
    }

    pub fn get_check_orthographies(&self, include: &[&str]) -> Vec<Orthography> {
        let include_all = include == OrthographyStatus::all();

        let orthographies: Vec<Orthography> = self
            .orthographies
            .iter()
            .filter(|o| include.contains(&o.status()))
            .cloned()
            .collect();

        if include_all {
            return orthographies;
        }

        let as_group: Vec<Orthography> = orthographies
            .iter()
            .filter(|o| o.preferred_as_group())
            .cloned()
            .collect();

        let as_individual: Vec<Orthography> = orthographies
            .iter()
            .filter(|o| !o.preferred_as_group())
            .cloned()
            .collect();

        let mut result = as_individual;

        if !as_group.is_empty() {
            // Combine character attributes across grouped orthographies
            let mut combined: HashMap<String, String> = HashMap::new();
            for ort in &as_group {
                for attr in CHARACTER_ATTRIBUTES {
                    if let Some(Value::String(s)) = ort.get(attr) {
                        if !s.is_empty() {
                            let entry = combined.entry(attr.to_string()).or_default();
                            if !entry.is_empty() {
                                entry.push(' ');
                            }
                            entry.push_str(s);
                        }
                    }
                }
            }

            for mut ort in as_group {
                for (attr, val) in &combined {
                    ort.data.insert(
                        Value::String(attr.clone()),
                        Value::String(val.clone()),
                    );
                }
                result.push(ort);
            }
        }

        result
    }
}
