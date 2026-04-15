use thiserror::Error;

#[derive(Error, Debug)]
pub enum HyperglotError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Language not found: {0}")]
    LanguageNotFound(String),
    #[error("No orthography found for script '{script}' status '{status}' in language '{lang}'")]
    OrthographyNotFound {
        script: String,
        status: String,
        lang: String,
    },
    #[error("Inheritance error: {0}")]
    Inheritance(String),
    #[error("Font error: {0}")]
    Font(String),
}

pub type Result<T> = std::result::Result<T, HyperglotError>;
