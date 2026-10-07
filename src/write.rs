//! Writing BibTeX back out in one consistent style.

use crate::bib::{Bib, Entry};

/// One entry: lower-case type and field names, two-space indent, the = signs
/// lined up, every value in braces and a comma after every field.
pub fn entry(e: &Entry) -> String {
    let width = e.fields.iter().map(|(n, _)| n.len()).max().unwrap_or(0);
    let mut out = format!("@{}{{{},\n", e.kind, e.key);
    for (name, value) in &e.fields {
        out += &format!("  {name:<width$} = {{{value}}},\n");
    }
    out += "}\n";
    out
}

/// The whole file, preambles first, a blank line between entries.
pub fn bib(b: &Bib) -> String {
    let mut parts: Vec<String> = b
        .preambles
        .iter()
        .map(|p| format!("@preamble{{{{{p}}}}}\n"))
        .collect();
    parts.extend(b.entries.iter().map(entry));
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bib::parse;

    const MESSY: &str = r#"
@string{jcp = "J. Comp. Phys."}
@ARTICLE( smith2020 , Title="A {"}quoted{"} title", JOURNAL = jcp,year=2020 ,
    pages = {1--10})
@book{doe, author = {Jane Doe}, title = {Nested {Braces {inside}}}}
"#;

    #[test]
    fn writes_the_house_style() {
        let out = bib(&parse(MESSY).unwrap());
        assert_eq!(
            out,
            "@article{smith2020,\n  title   = {A {\"}quoted{\"} title},\n  journal = {J. Comp. Phys.},\n  year    = {2020},\n  pages   = {1--10},\n}\n\n@book{doe,\n  author = {Jane Doe},\n  title  = {Nested {Braces {inside}}},\n}\n"
        );
    }

    #[test]
    fn output_reads_back_the_same() {
        let first = parse(MESSY).unwrap();
        let again = parse(&bib(&first)).unwrap();
        assert_eq!(first, again);
    }

    #[test]
    fn preamble_survives() {
        let first = parse("@preamble{\"\\providecommand{\\x}{}\"}\n@misc{a, note={n}}").unwrap();
        let out = bib(&first);
        assert!(
            out.starts_with("@preamble{{\\providecommand{\\x}{}}}\n"),
            "{out}"
        );
        assert_eq!(parse(&out).unwrap(), first);
    }
}
