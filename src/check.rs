//! Problems worth fixing before a bibliography goes into a paper.

use crate::bib::Entry;

/// Fields BibTeX's standard styles need for each entry type. "a|b" means
/// either one will do; biblatex's date counts as a year.
const REQUIRED: &[(&str, &[&str])] = &[
    ("article", &["author", "title", "journal", "year|date"]),
    (
        "book",
        &["author|editor", "title", "publisher", "year|date"],
    ),
    ("booklet", &["title"]),
    (
        "inbook",
        &[
            "author|editor",
            "title",
            "chapter|pages",
            "publisher",
            "year|date",
        ],
    ),
    (
        "incollection",
        &["author", "title", "booktitle", "publisher", "year|date"],
    ),
    (
        "inproceedings",
        &["author", "title", "booktitle", "year|date"],
    ),
    ("conference", &["author", "title", "booktitle", "year|date"]),
    ("manual", &["title"]),
    ("mastersthesis", &["author", "title", "school", "year|date"]),
    ("phdthesis", &["author", "title", "school", "year|date"]),
    ("proceedings", &["title", "year|date"]),
    (
        "techreport",
        &["author", "title", "institution", "year|date"],
    ),
    ("unpublished", &["author", "title", "note"]),
    ("misc", &[]),
    // biblatex types that are common in the wild.
    ("online", &["title", "url|doi"]),
    ("report", &["author", "title", "institution", "year|date"]),
    (
        "thesis",
        &["author", "title", "institution|school", "year|date"],
    ),
    ("software", &["title"]),
    ("dataset", &["title"]),
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
    let mut say = |message: String| {
        out.push(Problem {
            key: e.key.clone(),
            message,
        })
    };
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
            say(format!(
                "doi \"{doi}\" is a link; the field wants just the 10.xxxx/... part"
            ));
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

/// Words in the title with capitals after the first letter, outside braces.
/// Most BibTeX styles lower-case titles, so "DNA" prints as "dna" unless it
/// is written {DNA}.
pub fn unprotected_caps(e: &Entry) -> Vec<Problem> {
    let Some(title) = e.get("title") else {
        return Vec::new();
    };
    let mut depth = 0;
    let mut math = false;
    let mut word = String::new();
    let mut found: Vec<String> = Vec::new();
    let mut finish = |word: &mut String| {
        if word.chars().skip(1).any(char::is_uppercase) && !found.contains(word) {
            found.push(word.clone());
        }
        word.clear();
    };
    let mut prev = ' ';
    for c in title.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            '$' if prev != '\\' => math = !math,
            _ => {}
        }
        // A command like \LaTeX or a letter inside braces or math is safe.
        if depth == 0 && !math && c.is_alphanumeric() && prev != '\\' {
            word.push(c);
        } else if depth == 0 && !math && (c.is_alphanumeric() || c == '\\') {
            word.clear();
        } else {
            finish(&mut word);
        }
        if c == '\\' {
            prev = '\\';
        } else if !c.is_alphabetic() {
            prev = c;
        }
    }
    finish(&mut word);
    if found.is_empty() {
        return Vec::new();
    }
    let braced: Vec<String> = found.iter().map(|w| format!("{{{w}}}")).collect();
    vec![Problem {
        key: e.key.clone(),
        message: format!(
            "title has capitals most styles will lower-case; write {}",
            braced.join(", ")
        ),
    }]
}

/// Keys used by more than one entry. BibTeX silently uses the first.
/// Keys are compared ignoring case, as BibTeX does.
pub fn duplicate_keys(entries: &[Entry]) -> Vec<Problem> {
    let mut out = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        let earlier = entries[..i]
            .iter()
            .any(|o| o.key.eq_ignore_ascii_case(&e.key));
        if earlier {
            out.push(Problem {
                key: e.key.clone(),
                message: "the key is used by an earlier entry too".into(),
            });
        }
    }
    out
}

/// Fields given more than once. BibTeX reports "repeated entry" and keeps
/// only the first, so a corrected title added below the old one is lost.
pub fn repeated_fields(e: &Entry) -> Vec<Problem> {
    let mut out = Vec::new();
    for (i, (name, _)) in e.fields.iter().enumerate() {
        let firsts = e.fields[..i].iter().filter(|(n, _)| n == name).count();
        if firsts == 1 {
            out.push(Problem {
                key: e.key.clone(),
                message: format!("{name} is given more than once; BibTeX uses the first"),
            });
        }
    }
    out
}

/// crossref targets that are missing, or that come before the entry. BibTeX
/// reads the file in one pass, so it only fills in fields from an entry
/// that is further down.
pub fn bad_crossrefs(entries: &[Entry]) -> Vec<Problem> {
    let mut out = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        let Some(target) = e.get("crossref").map(str::trim) else {
            continue;
        };
        let at = entries
            .iter()
            .position(|o| o.key.eq_ignore_ascii_case(target));
        let message = match at {
            None => format!("crossref {target} isn't in the file"),
            Some(j) if j < i => {
                format!("crossref {target} comes before this entry; BibTeX needs it after")
            }
            Some(_) => continue,
        };
        out.push(Problem {
            key: e.key.clone(),
            message,
        });
    }
    out
}

/// Every check, in file order.
pub fn all(entries: &[Entry], caps: bool) -> Vec<Problem> {
    let mut out = duplicate_keys(entries);
    out.extend(bad_crossrefs(entries));
    for e in entries {
        out.extend(repeated_fields(e));
        out.extend(missing_fields(e));
        out.extend(bad_values(e));
        if caps {
            out.extend(unprotected_caps(e));
        }
    }
    let order = |key: &str| entries.iter().position(|e| e.key == key);
    out.sort_by_key(|p| order(&p.key));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bib::parse;

    fn entry(src: &str) -> Entry {
        parse(src).unwrap().entries.remove(0)
    }

    fn messages(problems: Vec<Problem>) -> Vec<String> {
        problems.into_iter().map(|p| p.message).collect()
    }

    #[test]
    fn crossrefs_must_exist_and_come_later() {
        let bib = parse(
            "@proceedings{early, title={P}}\n\
             @inproceedings{a, crossref={early}}\n\
             @inproceedings{b, crossref={Late}}\n\
             @inproceedings{c, crossref={gone}}\n\
             @proceedings{late, title={Q}}",
        )
        .unwrap();
        assert_eq!(
            messages(bad_crossrefs(&bib.entries)),
            [
                "crossref early comes before this entry; BibTeX needs it after",
                "crossref gone isn't in the file",
            ]
        );
    }

    #[test]
    fn repeated_fields_are_named_once() {
        let e = entry("@misc{a, title={One}, Title={Two}, note={n}, title={Three}}");
        assert_eq!(
            messages(repeated_fields(&e)),
            ["title is given more than once; BibTeX uses the first"]
        );
    }

    #[test]
    fn missing_fields_and_alternatives() {
        let e = entry("@article{a, author={X}, title={T}}");
        assert_eq!(
            messages(missing_fields(&e)),
            ["missing journal", "missing year or date"]
        );
        let e = entry("@article{a2, author={X}, title={T}, journal={J}, date={2021-03}}");
        assert!(missing_fields(&e).is_empty());
        let e = entry("@online{o, title={T}, url={https://example.org}}");
        assert!(missing_fields(&e).is_empty());
        let e = entry("@book{b, editor={E}, title={T}, publisher={P}, year=2000}");
        assert!(missing_fields(&e).is_empty());
        let e = entry("@book{c, title={T}, publisher={P}, year=2000}");
        assert_eq!(messages(missing_fields(&e)), ["missing author or editor"]);
        let e = entry("@article{d, author={ }, title={T}, journal={J}, year=1}");
        assert_eq!(messages(missing_fields(&e)), ["missing author"]);
        let e = entry("@webpage{w, title={T}}");
        assert_eq!(
            messages(missing_fields(&e)),
            ["@webpage isn't a standard entry type"]
        );
    }

    #[test]
    fn values_that_look_wrong() {
        let e = entry(
            "@misc{a, year={99}, pages={12-34}, doi={https://doi.org/10.1000/xyz}, url={http://a b}}",
        );
        assert_eq!(
            messages(bad_values(&e)),
            [
                "year \"99\" isn't a four-digit year",
                "pages \"12-34\" should use -- for a range",
                "doi \"https://doi.org/10.1000/xyz\" is a link; the field wants just the 10.xxxx/... part",
                "url has spaces in it",
            ]
        );
        let e = entry(
            "@misc{b, year={2001}, pages={12--34}, doi={10.1000/xyz}, url={https://x.org/a}}",
        );
        assert!(bad_values(&e).is_empty());
        let e = entry("@misc{c, pages={e1-e9}}");
        assert!(
            bad_values(&e).is_empty(),
            "article numbers like e1 aren't digit ranges"
        );
    }

    #[test]
    fn capitals_in_titles() {
        let e =
            entry(r"@misc{a, title={Sequencing DNA on GPUs with {CRISPR} and $O(N)$ in \LaTeX}}");
        assert_eq!(
            messages(unprotected_caps(&e)),
            ["title has capitals most styles will lower-case; write {DNA}, {GPUs}"]
        );
        let e = entry(r"@misc{b, title={A Plain Title With Initial Capitals}}");
        assert!(unprotected_caps(&e).is_empty());
    }

    #[test]
    fn duplicate_keys_ignore_case() {
        let bib = parse("@misc{Smith, note={a}}\n@misc{jones, note={b}}\n@misc{smith, note={c}}")
            .unwrap();
        let p = duplicate_keys(&bib.entries);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].key, "smith");
    }
}
