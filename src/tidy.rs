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
}
