//! Citation keys in one pattern: lastname, year, first real title word.

use crate::bib::Entry;
use crate::names::split_authors;
use crate::text::fold;

const SMALL_WORDS: [&str; 14] = ["a", "an", "the", "on", "of", "in", "for", "and", "to", "with", "at", "by", "from", "is"];

/// The key this entry would get on its own, like "knuth1984literate". Uses
/// the editor when there is no author, and leaves out parts that are missing.
pub fn suggest(e: &Entry) -> String {
    let people = e.get("author").or_else(|| e.get("editor")).unwrap_or("");
    let who = split_authors(people)
        .first()
        .map(|n| fold(&n.last).replace(' ', ""))
        .unwrap_or_default();
    let year: String = e.get("year").unwrap_or("").chars().filter(char::is_ascii_digit).collect();
    let title = fold(e.get("title").unwrap_or(""));
    let word = title.split(' ').find(|w| !w.is_empty() && !SMALL_WORDS.contains(w)).unwrap_or("");
    let key = format!("{who}{year}{word}");
    if key.is_empty() { e.key.clone() } else { key }
}
