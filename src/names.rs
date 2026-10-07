//! Splitting BibTeX author and editor lists into names.

/// One person. `von` holds lower-case particles like "van der" or "de".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Name {
    pub first: String,
    pub von: String,
    pub last: String,
    pub jr: String,
}

/// Splits on " and " outside braces, so {Barnes and Noble} stays one name.
/// A final "and others" is dropped; it only means "et al.".
pub fn split_authors(field: &str) -> Vec<Name> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let bytes = field.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            c if depth == 0 && c.is_ascii_whitespace() => {
                let rest = &field[i..];
                let word_end = rest.trim_start().find(|c: char| c.is_whitespace());
                let lead = rest.len() - rest.trim_start().len();
                if let Some(end) = word_end {
                    if rest.trim_start()[..end].eq_ignore_ascii_case("and") {
                        parts.push(&field[start..i]);
                        start = i + lead + end;
                        i = start;
                        continue;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&field[start..]);
    parts
        .into_iter()
        .map(str::trim)
        .filter(|p| !p.is_empty() && *p != "others")
        .map(parse_name)
        .collect()
}

/// Words split on spaces outside braces.
fn words(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut start = None;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
        if c.is_whitespace() && depth == 0 {
            if let Some(st) = start.take() {
                out.push(&s[st..i]);
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(st) = start {
        out.push(&s[st..]);
    }
    out
}

/// A von particle starts with a lower-case letter outside braces.
fn is_von(word: &str) -> bool {
    word.chars().next().is_some_and(|c| c.is_lowercase())
}

fn split_von_last(words: &[&str]) -> (String, String) {
    // The last word is always part of the last name, even if lower case.
    let Some((&last, before)) = words.split_last() else {
        return (String::new(), String::new());
    };
    let von_start = before.iter().position(|w| is_von(w));
    match von_start {
        Some(v) => {
            // von runs to the last lower-case word; capitalised words after it
            // belong to the last name ("de la Fontaine", "van Beethoven").
            let von_end = before.iter().rposition(|w| is_von(w)).unwrap() + 1;
            let mut last_words: Vec<&str> = before[von_end..].to_vec();
            last_words.push(last);
            (before[v..von_end].join(" "), last_words.join(" "))
        }
        None => (String::new(), last.to_string()),
    }
}

/// Reads "First von Last", "von Last, First" or "von Last, Jr, First".
pub fn parse_name(s: &str) -> Name {
    let commas: Vec<&str> = split_commas(s);
    match commas.as_slice() {
        [whole] => {
            let w = words(whole);
            // First names run up to the first von word, or to the last word.
            let first_end = w.iter().position(|x| is_von(x)).unwrap_or(w.len().saturating_sub(1));
            let (von, last) = split_von_last(&w[first_end..]);
            Name { first: w[..first_end].join(" "), von, last, jr: String::new() }
        }
        [last_part, first] => {
            let (von, last) = von_last_from(last_part);
            Name { first: first.trim().into(), von, last, jr: String::new() }
        }
        [last_part, jr, first, ..] => {
            let (von, last) = von_last_from(last_part);
            Name { first: first.trim().into(), von, last, jr: jr.trim().into() }
        }
        [] => Name::default(),
    }
}

fn von_last_from(s: &str) -> (String, String) {
    let w = words(s);
    // In "von Last, First" every leading lower-case word is von.
    let von_end = w.iter().take(w.len().saturating_sub(1)).take_while(|x| is_von(x)).count();
    (w[..von_end].join(" "), w[von_end..].join(" "))
}

fn split_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}
