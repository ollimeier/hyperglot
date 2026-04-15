pub const MARK_BASE: char = '◌'; // U+25CC
pub const CHARACTER_ATTRIBUTES: &[&str] = &["base", "auxiliary", "numerals", "punctuation", "currency"];
pub const VERSION: &str = "0.8.1";

#[derive(Debug, Clone, PartialEq)]
pub enum SupportLevel {
    Base,
    Aux,
    Punctuation,
    Numerals,
    Currency,
    All,
}

impl SupportLevel {
    pub fn values() -> Vec<&'static str> {
        vec!["base", "auxiliary", "punctuation", "numerals", "currency", "all"]
    }

    pub fn all() -> Vec<&'static str> {
        vec!["base", "auxiliary", "punctuation", "numerals", "currency"]
    }

    pub fn parse(input: &[String]) -> Vec<String> {
        if input.iter().any(|s| s == "all") {
            return Self::all().iter().map(|s| s.to_string()).collect();
        }
        input.to_vec()
    }

    pub fn value(&self) -> &'static str {
        match self {
            SupportLevel::Base => "base",
            SupportLevel::Aux => "auxiliary",
            SupportLevel::Punctuation => "punctuation",
            SupportLevel::Numerals => "numerals",
            SupportLevel::Currency => "currency",
            SupportLevel::All => "all",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LanguageValidity {
    Todo,
    Draft,
    Preliminary,
    Verified,
}

impl LanguageValidity {
    pub fn values() -> Vec<&'static str> {
        vec!["todo", "draft", "preliminary", "verified"]
    }

    pub fn index(val: &str) -> usize {
        Self::values().iter().position(|&v| v == val).unwrap_or(0)
    }

    pub fn value(&self) -> &'static str {
        match self {
            LanguageValidity::Todo => "todo",
            LanguageValidity::Draft => "draft",
            LanguageValidity::Preliminary => "preliminary",
            LanguageValidity::Verified => "verified",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LanguageStatus {
    Living,
    Historical,
    Constructed,
    All,
}

impl LanguageStatus {
    pub fn values() -> Vec<&'static str> {
        vec!["living", "historical", "constructed", "all"]
    }

    pub fn all() -> Vec<&'static str> {
        vec!["living", "historical", "constructed"]
    }

    pub fn parse(input: &[String]) -> Vec<String> {
        if input.iter().any(|s| s == "all") {
            return Self::all().iter().map(|s| s.to_string()).collect();
        }
        input.to_vec()
    }

    pub fn value(&self) -> &'static str {
        match self {
            LanguageStatus::Living => "living",
            LanguageStatus::Historical => "historical",
            LanguageStatus::Constructed => "constructed",
            LanguageStatus::All => "all",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrthographyStatus {
    Primary,
    Local,
    Secondary,
    Historical,
    Transliteration,
    All,
}

impl OrthographyStatus {
    pub fn values() -> Vec<&'static str> {
        vec!["primary", "local", "secondary", "historical", "transliteration", "all"]
    }

    pub fn index(val: &str) -> usize {
        Self::values().iter().position(|&v| v == val).unwrap_or(999)
    }

    pub fn all() -> Vec<&'static str> {
        vec!["primary", "local", "secondary", "historical", "transliteration"]
    }

    pub fn parse(input: &[String]) -> Vec<String> {
        if input.iter().any(|s| s == "all") {
            return Self::all().iter().map(|s| s.to_string()).collect();
        }
        input.to_vec()
    }

    pub fn value(&self) -> &'static str {
        match self {
            OrthographyStatus::Primary => "primary",
            OrthographyStatus::Local => "local",
            OrthographyStatus::Secondary => "secondary",
            OrthographyStatus::Historical => "historical",
            OrthographyStatus::Transliteration => "transliteration",
            OrthographyStatus::All => "all",
        }
    }
}
