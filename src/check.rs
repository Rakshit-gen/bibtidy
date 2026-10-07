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
