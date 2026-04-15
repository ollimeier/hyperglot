use std::collections::{HashMap, HashSet};

use serde_yaml::{Mapping, Value};

use crate::enums::OrthographyStatus;
use crate::error::{HyperglotError, Result};
use crate::parse::{character_list_from_string, filter_chars, list_unique, parse_chars, parse_marks};

/// Type alias for a function that can load an Orthography from (iso, script, status, attribute)
pub type LoadOrtFn<'a> = dyn Fn(&str, Option<&str>, Option<&str>) -> Result<Orthography> + 'a;

pub static INHERITABLE_DEFAULTS: &[&str] = &[
    "base",
    "auxiliary",
    "marks",
    "punctuation",
    "numerals",
    "currency",
    "combinations",
    "design_requirements",
];

#[derive(Debug, Clone)]
pub struct Orthography {
    pub data: Mapping,
}

impl Orthography {
    /// Create with defaults applied and optionally resolve inheritance.
    pub fn new(data: Mapping, expand: bool, load_ort: Option<&LoadOrtFn>) -> Result<Self> {
        let mut m = Mapping::new();

        if expand {
            // Apply defaults
            let defaults: &[(&str, Value)] = &[
                ("preferred_as_group", Value::Bool(false)),
                ("script_iso", Value::Null),
                ("autonym", Value::String(String::new())),
                ("script", Value::String(String::new())),
                (
                    "status",
                    Value::String(OrthographyStatus::Primary.value().to_string()),
                ),
                ("design_requirements", Value::Sequence(vec![])),
                ("combinations", Value::Sequence(vec![])),
            ];
            for (k, v) in defaults {
                m.insert(Value::String(k.to_string()), v.clone());
            }
            // inheritable defaults
            let inh_defaults: &[(&str, Value)] = &[
                ("base", Value::String(String::new())),
                ("auxiliary", Value::String(String::new())),
                ("marks", Value::String(String::new())),
                ("punctuation", Value::String(String::new())),
                ("numerals", Value::String(String::new())),
                ("currency", Value::String(String::new())),
                ("combinations", Value::String(String::new())),
                ("design_requirements", Value::Sequence(vec![])),
            ];
            for (k, v) in inh_defaults {
                m.insert(Value::String(k.to_string()), v.clone());
            }
        }

        // Override with actual data
        for (k, v) in &data {
            m.insert(k.clone(), v.clone());
        }

        let mut ort = Orthography { data: m };

        if expand {
            if let Some(loader) = load_ort {
                for attr in INHERITABLE_DEFAULTS {
                    ort.resolve_inherited_attr(attr, loader)?;
                }
            }
        }

        Ok(ort)
    }

    fn resolve_inherited_attr(&mut self, attr: &str, loader: &LoadOrtFn) -> Result<()> {
        let value = match self.data.get(Value::String(attr.to_string())) {
            Some(v) => v.clone(),
            None => return Ok(()),
        };

        match &value {
            Value::String(s) if !s.is_empty() => {
                let script = self.script().to_string();
                let resolved = resolve_inherited_attributes(s, attr, &script, loader)?;
                self.data.insert(
                    Value::String(attr.to_string()),
                    Value::String(resolved),
                );
            }
            Value::Sequence(list) => {
                let script = self.script().to_string();
                let mut new_list = Vec::new();
                for item in list.clone() {
                    if let Value::String(s) = item {
                        let resolved =
                            resolve_inherited_attributes(&s, attr, &script, loader)?;
                        new_list.push(Value::String(resolved));
                    } else {
                        new_list.push(item);
                    }
                }
                self.data.insert(
                    Value::String(attr.to_string()),
                    Value::Sequence(new_list),
                );
            }
            _ => {}
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(Value::String(key.to_string()))
    }

    pub fn get_str(&self, key: &str) -> &str {
        match self.get(key) {
            Some(Value::String(s)) => s.as_str(),
            _ => "",
        }
    }

    pub fn script(&self) -> &str {
        self.get_str("script")
    }

    pub fn status(&self) -> &str {
        let s = self.get_str("status");
        if s.is_empty() {
            OrthographyStatus::Primary.value()
        } else {
            s
        }
    }

    pub fn preferred_as_group(&self) -> bool {
        match self.get("preferred_as_group") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        }
    }

    fn character_list(&self, attr: &str) -> Vec<String> {
        match self.get(attr) {
            Some(Value::String(s)) if !s.is_empty() => {
                parse_chars(s, false, false)
            }
            _ => vec![],
        }
    }

    pub fn base(&self) -> Vec<String> {
        self.character_list("base")
    }

    pub fn base_chars(&self) -> Vec<String> {
        list_unique(
            self.character_list("base")
                .iter()
                .map(|c| filter_chars(c))
                .collect(),
        )
    }

    pub fn auxiliary(&self) -> Vec<String> {
        self.character_list("auxiliary")
    }

    pub fn auxiliary_chars(&self) -> Vec<String> {
        list_unique(
            self.character_list("auxiliary")
                .iter()
                .map(|c| filter_chars(c))
                .collect(),
        )
    }

    pub fn base_marks(&self) -> Vec<String> {
        self.all_marks("base")
    }

    pub fn auxiliary_marks(&self) -> Vec<String> {
        self.all_marks("aux")
    }

    pub fn required_base_marks(&self) -> Vec<String> {
        self.required_marks("base")
    }

    pub fn required_auxiliary_marks(&self) -> Vec<String> {
        self.required_marks("aux")
    }

    fn get_raw_str(&self, key: &str) -> &str {
        match self.data.get(Value::String(key.to_string())) {
            Some(Value::String(s)) => s.as_str(),
            _ => "",
        }
    }

    fn all_marks(&self, level: &str) -> Vec<String> {
        let marks = parse_marks(self.get_raw_str("marks"), true);
        let decom_base = parse_marks(self.get_raw_str("base"), true);
        let decom_aux = parse_marks(self.get_raw_str("auxiliary"), true);

        if level == "base" {
            let only_aux: Vec<&String> = decom_aux
                .iter()
                .filter(|m| !decom_base.contains(m))
                .collect();
            let combined: Vec<String> = marks
                .iter()
                .chain(decom_base.iter())
                .filter(|m| !only_aux.contains(m))
                .cloned()
                .collect();
            return list_unique(combined);
        }

        if level == "aux" {
            let has_aux = !self.get_raw_str("auxiliary").is_empty();
            if has_aux {
                return list_unique(
                    marks
                        .into_iter()
                        .chain(decom_base)
                        .chain(decom_aux)
                        .collect(),
                );
            } else {
                return list_unique(marks.into_iter().chain(decom_base).collect());
            }
        }

        vec![]
    }

    fn required_marks(&self, level: &str) -> Vec<String> {
        let marks = parse_marks(self.get_raw_str("marks"), true);
        let marks_base = parse_marks(self.get_raw_str("base"), true);
        let marks_aux = parse_marks(self.get_raw_str("auxiliary"), true);

        let non_decomposable: Vec<String> = marks
            .iter()
            .filter(|m| !marks_base.contains(m) && !marks_aux.contains(m))
            .cloned()
            .collect();

        let mut marks_unencoded = Vec::new();
        let base_raw = self.get_raw_str("base");
        for c in character_list_from_string(base_raw, true) {
            if c.chars().count() > 1 {
                marks_unencoded.extend(parse_marks(&c, true));
            }
        }

        if level == "aux" {
            let aux_raw = self.get_raw_str("auxiliary");
            for c in character_list_from_string(aux_raw, true) {
                if c.chars().count() > 1 {
                    marks_unencoded.extend(parse_marks(&c, true));
                }
            }
        }

        list_unique(non_decomposable.into_iter().chain(marks_unencoded).collect())
    }

    pub fn get_chars(&self, attr: &str, all_marks: bool) -> HashSet<String> {
        if attr == "aux" {
            let chars = self.auxiliary_chars();
            let marks = if all_marks {
                self.auxiliary_marks()
            } else {
                self.required_auxiliary_marks()
            };
            return chars.into_iter().chain(marks).collect();
        }
        let chars = self.base_chars();
        let marks = if all_marks {
            self.base_marks()
        } else {
            self.required_base_marks()
        };
        chars.into_iter().chain(marks).collect()
    }

    pub fn combinations(&self) -> HashMap<String, f64> {
        let mut d = HashMap::new();
        if let Some(Value::Sequence(list)) = self.get("combinations") {
            for item in list {
                if let Value::Mapping(map) = item {
                    for (k, v) in map {
                        if let (Value::String(key), freq) = (k, v) {
                            let freq_val = match freq {
                                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                                _ => 0.0,
                            };
                            d.insert(key.clone(), freq_val);
                        }
                    }
                }
            }
        }
        d
    }

    pub fn numerals(&self) -> Vec<String> {
        self.character_list("numerals")
    }

    pub fn punctuation(&self) -> Vec<String> {
        self.character_list("punctuation")
    }

    pub fn currency(&self) -> Vec<String> {
        self.character_list("currency")
    }
}

/// Extract ISO/script/status/attribute from an inheritance code
fn extract_inheritance_specifics(
    code: &str,
    attr: &str,
    _loader: &LoadOrtFn,
) -> Result<(String, String, Option<String>, Option<String>)> {
    let code = code.trim().trim_start_matches('<').trim_end_matches('>').trim();
    let parts: Vec<&str> = code.split_whitespace().collect();
    if parts.is_empty() {
        return Err(HyperglotError::Inheritance(format!(
            "Empty inheritance code: {}",
            code
        )));
    }

    let iso = parts[0].to_string();
    if iso.len() < 3 && iso != "default" {
        return Err(HyperglotError::Inheritance(format!(
            "Not a valid iso code: {}",
            iso
        )));
    }

    let mut attribute = attr.to_string();
    let mut status: Option<String> = None;
    let mut script: Option<String> = None;

    // Try to load scripts data to check script names
    let scripts_data = crate::loader::load_scripts_data().unwrap_or_default();

    for p in &parts[1..] {
        if INHERITABLE_DEFAULTS.contains(p) {
            attribute = p.to_string();
        } else if let Some(Value::Mapping(_)) =
            scripts_data.get(Value::String(p.to_string()))
        {
            script = Some(p.to_string());
        } else if OrthographyStatus::values().contains(p) {
            status = Some(p.to_string());
        }
    }

    Ok((iso, attribute, status, script))
}

/// Find all inheritance codes in a value string
fn find_all_inheritance_codes(value: &str) -> Vec<(String, usize, usize)> {
    use once_cell::sync::Lazy;
    use regex::Regex;
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<([A-Za-zʽ' ]*)>").unwrap());

    let mut result = Vec::new();
    for cap in RE.captures_iter(value) {
        let inner = cap[1].trim();
        if inner.len() < 3 {
            continue;
        }
        let m = cap.get(0).unwrap();
        result.push((inner.to_string(), m.start(), m.len()));
    }
    result
}

fn resolve_inherited_attributes(
    value: &str,
    attr: &str,
    script: &str,
    loader: &LoadOrtFn,
) -> Result<String> {
    let codes = find_all_inheritance_codes(value);
    if codes.is_empty() {
        return Ok(value.to_string());
    }

    let mut resolved = value.to_string();

    for (code, beginning, length) in codes {
        let (iso, attribute, status, _script) =
            extract_inheritance_specifics(&code, attr, loader)?;

        let effective_script = _script.as_deref().or(if script.is_empty() {
            None
        } else {
            Some(script)
        });

        let ort = loader(&iso, effective_script, status.as_deref())?;

        let replacement = match ort.get(&attribute) {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Sequence(seq)) => {
                // Convert sequence to string representation
                seq.iter()
                    .filter_map(|v| {
                        if let Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            _ => {
                return Err(HyperglotError::Inheritance(format!(
                    "Cannot inherit '{}' from '{}'",
                    attribute, iso
                )));
            }
        };

        // Recursively resolve if needed
        let replacement = if replacement.contains('<') {
            resolve_inherited_attributes(&replacement, &attribute, script, loader)?
        } else {
            replacement
        };

        let before = if beginning > 0 {
            resolved[..beginning - 1].to_string()
        } else {
            String::new()
        };
        let after_start = beginning + length + 1;
        let after = if after_start <= resolved.len() {
            resolved[after_start..].to_string()
        } else {
            String::new()
        };

        resolved = format!("{} {} {}", before, replacement, after)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Re-check for more codes
        let new_codes = find_all_inheritance_codes(&resolved);
        if new_codes.is_empty() {
            break;
        }
    }

    Ok(resolved)
}
