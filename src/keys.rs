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
        // "van Dyke" is the surname, so the von part stays in.
        .map(|n| fold(&format!("{} {}", n.von, n.last)).replace(' ', ""))
        .unwrap_or_default();
    let year: String = e.get("year").unwrap_or("").chars().filter(char::is_ascii_digit).collect();
    let title = fold(e.get("title").unwrap_or(""));
    let word = title.split(' ').find(|w| !w.is_empty() && !SMALL_WORDS.contains(w)).unwrap_or("");
    let key = format!("{who}{year}{word}");
    if key.is_empty() { e.key.clone() } else { key }
}

/// New keys for every entry, in file order. When two entries would get the
/// same key, they all get a letter on the end in file order: smith2020a,
/// smith2020b.
pub fn assign(entries: &[Entry]) -> Vec<String> {
    let wanted: Vec<String> = entries.iter().map(suggest).collect();
    let mut seen: Vec<(&str, usize)> = Vec::new();
    wanted
        .iter()
        .map(|k| {
            let total = wanted.iter().filter(|w| *w == k).count();
            if total == 1 {
                return k.clone();
            }
            let n = match seen.iter_mut().find(|(s, _)| s == k) {
                Some((_, n)) => {
                    *n += 1;
                    *n
                }
                None => {
                    seen.push((k, 0));
                    0
                }
            };
            format!("{k}{}", suffix(n))
        })
        .collect()
}

/// a, b, ..., z, aa, ab, ...
fn suffix(mut n: usize) -> String {
    let mut s = String::new();
    loop {
        s.insert(0, (b'a' + (n % 26) as u8) as char);
        if n < 26 {
            return s;
        }
        n = n / 26 - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bib::parse;

    fn keys(src: &str) -> Vec<String> {
        assign(&parse(src).unwrap().entries)
    }

    #[test]
    fn the_usual_pattern() {
        let k = keys(r#"@book{x, author={Donald E. Knuth}, title={The {\TeX}book}, year={1984}}"#);
        assert_eq!(k, ["knuth1984texbook"]);
        let k = keys(r#"@article{x, author={G\"{o}del, Kurt and Other, A.}, title={On Formally Undecidable Propositions}, year=1931}"#);
        assert_eq!(k, ["godel1931formally"]);
    }

    #[test]
    fn falls_back_when_parts_are_missing() {
        assert_eq!(keys("@misc{x, editor={Ann van Dyke}, title={Notes}}"), ["vandykenotes"]);
        assert_eq!(keys("@misc{keepme, note={nothing to go on}}"), ["keepme"]);
    }

    #[test]
    fn clashes_get_letters() {
        let k = keys(
            "@misc{1, author={Smith, J.}, title={Data}, year=2020}\n@misc{2, author={Jones, K.}, title={Data}, year=2020}\n@misc{3, author={Smith, A.}, title={Data!}, year=2020}",
        );
        assert_eq!(k, ["smith2020dataa", "jones2020data", "smith2020datab"]);
        assert_eq!(suffix(0), "a");
        assert_eq!(suffix(25), "z");
        assert_eq!(suffix(26), "aa");
    }
}
