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
    let mut parts: Vec<String> = b.preambles.iter().map(|p| format!("@preamble{{{{{p}}}}}\n")).collect();
    parts.extend(b.entries.iter().map(entry));
    parts.join("\n")
}
