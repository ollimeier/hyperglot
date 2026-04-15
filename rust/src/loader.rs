use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde_yaml::Value;

use crate::error::{HyperglotError, Result};

static YAML_CACHE: Lazy<Mutex<HashMap<String, Value>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("HYPERGLOT_DATA_PATH") {
        PathBuf::from(p)
    } else {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../lib/hyperglot/data"
        ))
    }
}

pub fn db_extra_path() -> PathBuf {
    if let Ok(p) = std::env::var("HYPERGLOT_EXTRA_DATA_PATH") {
        PathBuf::from(p)
    } else {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../lib/hyperglot/extra_data"
        ))
    }
}

pub fn load_cached_yaml(path: &PathBuf) -> Result<Value> {
    let key = path.to_string_lossy().to_string();
    {
        let cache = YAML_CACHE.lock().unwrap();
        if let Some(v) = cache.get(&key) {
            return Ok(v.clone());
        }
    }
    let content = std::fs::read_to_string(path)?;
    let value: Value = serde_yaml::from_str(&content)?;
    {
        let mut cache = YAML_CACHE.lock().unwrap();
        cache.insert(key, value.clone());
    }
    Ok(value)
}

pub fn load_language_data(iso: &str) -> Result<Value> {
    let base = if iso == "default" {
        db_extra_path()
    } else {
        db_path()
    };

    let candidates = [
        format!("{}.yaml", iso),
        format!("{}_.yaml", iso),
        format!("{}.yml", iso),
        format!("{}_.yml", iso),
    ];

    for candidate in &candidates {
        let path = base.join(candidate.trim());
        if path.exists() {
            return load_cached_yaml(&path);
        }
    }

    Err(HyperglotError::LanguageNotFound(iso.to_string()))
}

pub fn load_scripts_data() -> Result<Value> {
    load_cached_yaml(&db_extra_path().join("script-names.yaml"))
}

pub fn load_joining_types() -> Result<HashMap<String, String>> {
    let val = load_cached_yaml(&db_extra_path().join("joining-types.yaml"))?;
    let mut map = HashMap::new();
    if let Value::Mapping(m) = val {
        for (k, v) in m {
            if let (Value::String(key), Value::String(val)) = (k, v) {
                map.insert(key, val);
            }
        }
    }
    Ok(map)
}
