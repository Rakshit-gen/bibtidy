//! Problems worth fixing before a bibliography goes into a paper.

use crate::bib::Entry;

/// Fields BibTeX's standard styles need for each entry type. "a|b" means
/// either one will do.
const REQUIRED: &[(&str, &[&str])] = &[
    ("article", &["author", "title", "journal", "year"]),
    ("book", &["author|editor", "title", "publisher", "year"]),
    ("booklet", &["title"]),
    ("inbook", &["author|editor", "title", "chapter|pages", "publisher", "year"]),
    ("incollection", &["author", "title", "booktitle", "publisher", "year"]),
    ("inproceedings", &["author", "title", "booktitle", "year"]),
    ("conference", &["author", "title", "booktitle", "year"]),
    ("manual", &["title"]),
    ("mastersthesis", &["author", "title", "school", "year"]),
    ("phdthesis", &["author", "title", "school", "year"]),
    ("proceedings", &["title", "year"]),
    ("techreport", &["author", "title", "institution", "year"]),
    ("unpublished", &["author", "title", "note"]),
    ("misc", &[]),
];

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub key: String,
    pub message: String,
}

fn has(e: &Entry, name: &str) -> bool {
    e.get(name).is_some_and(|v| !v.trim().is_empty())
}

/// Required fields that are missing or empty.
pub fn missing_fields(e: &Entry) -> Vec<Problem> {
    let Some((_, needed)) = REQUIRED.iter().find(|(k, _)| *k == e.kind) else {
        return vec![Problem {
            key: e.key.clone(),
            message: format!("@{} isn't a standard entry type", e.kind),
        }];
    };
    needed
        .iter()
        .filter(|alts| !alts.split('|').any(|f| has(e, f)))
        .map(|alts| Problem {
            key: e.key.clone(),
            message: format!("missing {}", alts.replace('|', " or ")),
        })
        .collect()
}

/// Values that are present but look wrong.
pub fn bad_values(e: &Entry) -> Vec<Problem> {
    let mut out = Vec::new();
    let mut say = |message: String| out.push(Problem { key: e.key.clone(), message });
    if let Some(year) = e.get("year") {
        let year = year.trim();
        if !(year.len() == 4 && year.chars().all(|c| c.is_ascii_digit())) {
            say(format!("year \"{year}\" isn't a four-digit year"));
        }
    }
    if let Some(pages) = e.get("pages") {
        // 12-34 prints as a hyphen; a page range wants an en dash, 12--34.
        let b = pages.as_bytes();
        let single = (1..b.len().saturating_sub(1))
            .any(|i| b[i] == b'-' && b[i - 1].is_ascii_digit() && b[i + 1].is_ascii_digit());
        if single {
            say(format!("pages \"{pages}\" should use -- for a range"));
        }
    }
    if let Some(doi) = e.get("doi") {
        let doi = doi.trim();
        if doi.contains("doi.org/") {
            say(format!("doi \"{doi}\" is a link; the field wants just the 10.xxxx/... part"));
        } else if !doi.starts_with("10.") {
            say(format!("doi \"{doi}\" doesn't start with 10."));
        }
    }
    if let Some(url) = e.get("url") {
        if url.trim().contains(char::is_whitespace) {
            say("url has spaces in it".into());
        }
    }
    out
}
