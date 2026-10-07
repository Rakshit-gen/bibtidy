//! Changes to a bibliography that don't alter what it cites.

use crate::bib::Entry;

/// The order fields are written in. Anything not listed keeps its original
/// order after these.
const FIELD_ORDER: [&str; 18] = [
    "author",
    "editor",
    "title",
    "booktitle",
    "journal",
    "series",
    "volume",
    "number",
    "chapter",
    "pages",
    "edition",
    "publisher",
    "organization",
    "institution",
    "school",
    "address",
    "year",
    "month",
];

/// Puts the fields in the usual reading order: who, what, where, when.
pub fn order_fields(e: &mut Entry) {
    let rank = |name: &str| FIELD_ORDER.iter().position(|&n| n == name).unwrap_or(FIELD_ORDER.len());
    // sort_by_key is stable, so unlisted fields stay in file order.
    e.fields.sort_by_key(|(name, _)| rank(name));
}

/// Fixes that can't change what gets printed except for the better:
/// runs of spaces and line breaks become one space (BibTeX treats them the
/// same), page ranges get --, and a DOI pasted as a link loses the link part.
/// Returns how many fields changed.
pub fn fix(e: &mut Entry) -> usize {
    let mut changed = 0;
    for (name, value) in &mut e.fields {
        let mut new = value.split_whitespace().collect::<Vec<_>>().join(" ");
        if name == "pages" {
            new = en_dash_ranges(&new);
        }
        if name == "doi" {
            if let Some((_, rest)) = new.rsplit_once("doi.org/") {
                new = rest.to_string();
            }
        }
        if new != *value {
            *value = new;
            changed += 1;
        }
    }
    changed
}

/// 12-34 and 12 - 34 become 12--34; e1-e9 is left alone.
fn en_dash_ranges(pages: &str) -> String {
    let compact = pages.replace(" - ", "-").replace(" -- ", "--");
    let c: Vec<char> = compact.chars().collect();
    let mut out = String::new();
    for (i, &ch) in c.iter().enumerate() {
        out.push(ch);
        let digits_around = i > 0 && c[i - 1].is_ascii_digit() && c.get(i + 1).is_some_and(|n| n.is_ascii_digit());
        if ch == '-' && digits_around {
            out.push('-');
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortBy {
    Key,
    /// Oldest first, entries without a year last, ties by key.
    Year,
}

pub fn sort_entries(entries: &mut [Entry], by: SortBy) {
    match by {
        SortBy::Key => entries.sort_by_key(|e| e.key.to_lowercase()),
        SortBy::Year => entries.sort_by_key(|e| {
            let year = e.get("year").and_then(|y| y.trim().parse::<i32>().ok());
            (year.is_none(), year, e.key.to_lowercase())
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bib::parse;

    #[test]
    fn orders_known_fields_and_keeps_the_rest_in_place() {
        let mut e = parse("@article{a, year={1}, doi={d}, title={t}, note={n}, author={x}}").unwrap().entries.remove(0);
        order_fields(&mut e);
        let names: Vec<_> = e.fields.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["author", "title", "year", "doi", "note"]);
    }

    #[test]
    fn sorts_by_key_or_year() {
        let mut bib = parse("@misc{b, year={2001}}\n@misc{A, year={1999}}\n@misc{c}\n@misc{d, year={1999}}").unwrap();
        sort_entries(&mut bib.entries, SortBy::Key);
        let keys: Vec<_> = bib.entries.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["A", "b", "c", "d"]);
        sort_entries(&mut bib.entries, SortBy::Year);
        let keys: Vec<_> = bib.entries.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["A", "d", "b", "c"]);
    }

    #[test]
    fn safe_fixes() {
        let mut e = parse("@misc{a, title={Two\n    lines}, pages={12 - 34}, doi={https://doi.org/10.1/x}, note={ok}}")
            .unwrap()
            .entries
            .remove(0);
        assert_eq!(fix(&mut e), 3);
        assert_eq!(e.get("title"), Some("Two lines"));
        assert_eq!(e.get("pages"), Some("12--34"));
        assert_eq!(e.get("doi"), Some("10.1/x"));
        assert_eq!(fix(&mut e), 0);
        assert_eq!(en_dash_ranges("e1-e9, 3-5"), "e1-e9, 3--5");
    }
}
