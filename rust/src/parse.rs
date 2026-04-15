use once_cell::sync::Lazy;
use regex::Regex;
use unicode_general_category::{get_general_category, GeneralCategory};
use unicode_normalization::UnicodeNormalization;

use crate::loader::load_joining_types;

pub const MARK_BASE_CHAR: char = '◌';

static RE_INHERITANCE_TAG: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<([A-Za-zʽ' ]*)>").unwrap());
static RE_MULTIPLE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s{2,}").unwrap());

pub fn list_unique(v: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for item in v {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

pub fn is_mark(c: char) -> bool {
    let cat = get_general_category(c);
    matches!(
        cat,
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
    )
}

pub fn character_list_from_string(s: &str, normalize: bool) -> Vec<String> {
    let input: String = if normalize {
        s.nfc().collect()
    } else {
        s.to_string()
    };

    // Remove all whitespace
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    let chars: Vec<char> = input.chars().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let mut end = i + 1;
        // Consume all following marks
        while end < chars.len() && is_mark(chars[end]) {
            end += 1;
        }
        let cluster: String = chars[i..end].iter().collect();
        result.push(cluster);
        i = end;
    }

    list_unique(result.into_iter().filter(|s| !s.trim().is_empty()).collect())
}

pub fn decompose_fully(c: char) -> Vec<char> {
    let s = c.to_string();
    let nfd: String = s.nfd().collect();
    let nfd_chars: Vec<char> = nfd.chars().collect();
    if nfd_chars == vec![c] {
        return vec![c];
    }
    nfd_chars.iter().flat_map(|&d| decompose_fully(d)).collect()
}

fn sort_key_character_category(c: char) -> (usize, u32) {
    use unicode_general_category::GeneralCategory::*;
    let cat = get_general_category(c);
    let order = match cat {
        UppercaseLetter => 0,
        TitlecaseLetter => 1,
        LowercaseLetter => 2,
        ModifierLetter | OtherLetter => 5,
        NonspacingMark => 6,
        EnclosingMark => 7,
        SpacingMark => 8,
        _ => 9,
    };
    (order, c as u32)
}

pub fn sort_by_character_type(mut chars: Vec<String>) -> Vec<String> {
    chars.sort_by(|a, b| {
        let a_key = a.chars().next().map(sort_key_character_category).unwrap_or((99, 0));
        let b_key = b.chars().next().map(sort_key_character_category).unwrap_or((99, 0));
        a_key.cmp(&b_key)
    });
    chars
}

pub fn parse_chars(characters: &str, decompose: bool, retain_decomposed: bool) -> Vec<String> {
    if !decompose {
        return character_list_from_string(characters, false);
    }

    let unique_strings = character_list_from_string(characters, true);
    let combined = unique_strings.join(" ");

    let mut unique_chars: Vec<String> = Vec::new();
    let mut additional: Vec<String> = Vec::new();

    for c in combined.chars() {
        if c.is_whitespace() {
            continue;
        }
        let decomposition = decompose_fully(c);
        if decomposition == vec![c] || retain_decomposed {
            unique_chars.push(c.to_string());
        }
        if decomposition != vec![c] {
            for d in &decomposition {
                additional.push(d.to_string());
            }
        }
    }

    additional = sort_by_character_type(additional);
    let mut all_chars = unique_chars;
    all_chars.extend(additional);

    let result = list_unique(all_chars);
    result.into_iter().filter(|s| {
        !s.chars().all(|c| c.is_whitespace()) && !s.is_empty()
    }).collect()
}

pub fn filter_chars(b: &str) -> String {
    if b.chars().count() > 1 {
        // Find first non-mark char from parse_chars
        for c_str in parse_chars(b, true, false) {
            if c_str.chars().count() == 1 {
                let c = c_str.chars().next().unwrap();
                if !is_mark(c) {
                    return c.to_string();
                }
            }
        }
        b.to_string()
    } else {
        b.to_string()
    }
}

pub fn remove_mark_base(input: &str) -> String {
    input.replace(MARK_BASE_CHAR, "")
}

pub fn parse_marks(input: &str, decompose: bool) -> Vec<String> {
    if input.is_empty() {
        return vec![];
    }
    let (pruned, _, _) = drop_inheritance_tags(input);
    let pruned = remove_mark_base(&pruned);
    let chars = parse_chars(&pruned, decompose, false);
    chars.into_iter().filter(|c| {
        c.chars().count() == 1 && c.chars().next().map(is_mark).unwrap_or(false)
    }).collect()
}

/// Returns (pruned_string, template_with_placeholders, vec_of_tags)
pub fn drop_inheritance_tags(input: &str) -> (String, String, Vec<String>) {
    if input.is_empty() {
        return (String::new(), String::new(), vec![]);
    }

    let tags: Vec<String> = RE_INHERITANCE_TAG
        .captures_iter(input)
        .map(|cap| format!("<{}>", RE_MULTIPLE_SPACES.replace_all(cap[1].trim(), " ")))
        .collect();

    if tags.is_empty() {
        return (input.to_string(), input.to_string(), vec![]);
    }

    let pruned = RE_INHERITANCE_TAG.replace_all(input, "").to_string();
    let pruned = RE_MULTIPLE_SPACES.replace_all(&pruned, " ").to_string();

    let inserts = RE_INHERITANCE_TAG.replace_all(input, "%s").to_string();
    let inserts = RE_MULTIPLE_SPACES.replace_all(&inserts, " ").to_string();

    (pruned, inserts, tags)
}

pub fn get_joining_type(c: char) -> String {
    let types = load_joining_types().unwrap_or_default();
    let key = c.to_string();
    types.get(&key).cloned().unwrap_or_default()
}

pub fn join_variants(c: char, joiner: char) -> Vec<String> {
    let t = get_joining_type(c);
    match t.as_str() {
        "R" => vec![format!("{}{}", joiner, c)],
        "D" => vec![
            format!("{}{}", joiner, c),
            format!("{}{}{}", joiner, c, joiner),
            format!("{}{}", c, joiner),
        ],
        "L" => vec![format!("{}{}", c, joiner)],
        _ => vec![],
    }
}
