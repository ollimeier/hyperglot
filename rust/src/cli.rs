use clap::Parser;
use std::collections::HashSet;

use crate::checker::{CharsetChecker, CheckOptions, FontChecker};

#[derive(Parser, Debug)]
#[command(name = "hyperglot", about = "Check font or character set language support")]
pub struct Cli {
    /// Font paths or character strings to check
    pub inputs: Vec<String>,

    /// Check level: base, auxiliary, punctuation, numerals, currency, all
    #[arg(long, default_value = "base")]
    pub check: Vec<String>,

    /// Minimum validity level
    #[arg(long, default_value = "draft")]
    pub validity: String,

    /// Language status filter
    #[arg(long, default_value = "living")]
    pub status: Vec<String>,

    /// Orthography status filter
    #[arg(long, default_value = "primary")]
    pub orthography: Vec<String>,

    /// Treat inputs as character sets, not fonts
    #[arg(long)]
    pub chars: bool,

    /// Include decomposed matches
    #[arg(long)]
    pub decomposed: bool,

    /// Require all marks
    #[arg(long)]
    pub marks: bool,
}

pub fn run() {
    env_logger::init();

    let cli = Cli::parse();

    let options = CheckOptions {
        check: cli.check,
        validity: cli.validity,
        status: cli.status,
        orthography: cli.orthography,
        decomposed: cli.decomposed,
        marks: cli.marks,
        shaping: !cli.chars,
        shaping_threshold: 0.01,
    };

    for input in &cli.inputs {
        println!("Checking: {}", input);

        if cli.chars {
            let chars: HashSet<String> = input.chars().map(|c| c.to_string()).collect();
            let checker = CharsetChecker::new(chars);
            match checker.get_supported_languages(options.clone()) {
                Ok(support) => print_support(&support),
                Err(e) => eprintln!("Error: {}", e),
            }
        } else {
            match FontChecker::new(input) {
                Ok(checker) => match checker.get_supported_languages(options.clone()) {
                    Ok(support) => print_support(&support),
                    Err(e) => eprintln!("Error: {}", e),
                },
                Err(e) => eprintln!("Error loading font '{}': {}", input, e),
            }
        }
    }
}

fn print_support(
    support: &std::collections::HashMap<
        String,
        std::collections::HashMap<String, crate::language::Language>,
    >,
) {
    if support.is_empty() {
        println!("No supported languages found.");
        return;
    }

    let mut scripts: Vec<&String> = support.keys().collect();
    scripts.sort();

    for script in scripts {
        let langs = &support[script];
        println!("\n{} ({} languages):", script, langs.len());
        let mut lang_names: Vec<String> = langs
            .iter()
            .map(|(iso, lang)| format!("{} ({})", lang.name(), iso))
            .collect();
        lang_names.sort();
        for name in lang_names {
            println!("  {}", name);
        }
    }
}
