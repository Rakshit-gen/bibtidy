//! Citation keys in one pattern: lastname, year, first real title word.

use crate::bib::Entry;
use crate::names::split_authors;
use crate::text::fold;

const SMALL_WORDS: [&str; 14] = [
    "a", "an", "the", "on", "of", "in", "for", "and", "to", "with", "at", "by", "from", "is",
];

/// The key this entry would get on its own, like "knuth1984literate". Uses
/// the editor when there is no author, and leaves out parts that are missing.
pub fn suggest(e: &Entry) -> String {
    let people = e.get("author").or_else(|| e.get("editor")).unwrap_or("");
    let who = split_authors(people)
        .first()
        // "van Dyke" is the surname, so the von part stays in.
        .map(|n| fold(&format!("{} {}", n.von, n.last)).replace(' ', ""))
        .unwrap_or_default();
    let year: String = e
        .get("year")
        .unwrap_or("")
        .chars()
        .filter(char::is_ascii_digit)
        .collect();
    let title = fold(e.get("title").unwrap_or(""));
    let word = title
        .split(' ')
        .find(|w| !w.is_empty() && !SMALL_WORDS.contains(w))
        .unwrap_or("");
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

/// Rewrites keys inside \\cite, \\citep, \\textcite, \\nocite and the other
/// commands with "cite" in their name, keeping [optional] arguments and
/// spacing. Returns the new text and how many keys changed.
/// Points crossref and xref fields at the new keys. Left alone they name a
/// key that no longer exists, and the entry loses everything it inherited,
/// like the booktitle of the proceedings it's in. Keys match ignoring case,
/// as in BibTeX.
pub fn rename_references(entries: &mut [Entry], renames: &[(String, String)]) -> usize {
    let mut n = 0;
    for e in entries {
        for (name, value) in &mut e.fields {
            if name != "crossref" && name != "xref" {
                continue;
            }
            let target = value.trim();
            if let Some((_, new)) = renames
                .iter()
                .find(|(old, _)| old.eq_ignore_ascii_case(target))
            {
                *value = new.clone();
                n += 1;
            }
        }
    }
    n
}

pub fn rename_citations(tex: &str, renames: &[(String, String)]) -> (String, usize) {
    let mut out = String::with_capacity(tex.len());
    let mut changed = 0;
    let mut rest = tex;
    while let Some(at) = rest.find('\\') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let name_len = rest[1..]
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(rest.len() - 1);
        let name = &rest[1..1 + name_len];
        if !name.contains("cite") {
            // The backslash and an ASCII name are single bytes each.
            out.push_str(&rest[..1 + name_len]);
            rest = &rest[1 + name_len..];
            continue;
        }
        // Copy the command, a star, and any [..] arguments as they are.
        let mut i = 1 + name_len;
        if rest[i..].starts_with('*') {
            i += 1;
        }
        loop {
            let ws = rest[i..].len() - rest[i..].trim_start().len();
            if rest[i + ws..].starts_with('[') {
                match rest[i + ws..].find(']') {
                    Some(close) => i += ws + close + 1,
                    None => break,
                }
            } else {
                break;
            }
        }
        let ws = rest[i..].len() - rest[i..].trim_start().len();
        let open = i + ws;
        let close = rest[open..].find('}').map(|c| open + c);
        match (rest[open..].starts_with('{'), close) {
            (true, Some(close)) => {
                out.push_str(&rest[..open + 1]);
                let keys: Vec<String> = rest[open + 1..close]
                    .split(',')
                    .map(|k| {
                        let trimmed = k.trim();
                        match renames.iter().find(|(old, _)| old == trimmed) {
                            Some((_, new)) => {
                                changed += 1;
                                k.replacen(trimmed, new, 1)
                            }
                            None => k.to_string(),
                        }
                    })
                    .collect();
                out.push_str(&keys.join(","));
                out.push('}');
                rest = &rest[close + 1..];
            }
            _ => {
                out.push_str(&rest[..i]);
                rest = &rest[i..];
            }
        }
    }
    out.push_str(rest);
    (out, changed)
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
        let k = keys(
            r#"@article{x, author={G\"{o}del, Kurt and Other, A.}, title={On Formally Undecidable Propositions}, year=1931}"#,
        );
        assert_eq!(k, ["godel1931formally"]);
    }

    #[test]
    fn falls_back_when_parts_are_missing() {
        assert_eq!(
            keys("@misc{x, editor={Ann van Dyke}, title={Notes}}"),
            ["vandykenotes"]
        );
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

    #[test]
    fn renames_crossrefs() {
        let mut bib = parse("@inproceedings{p, crossref={Procs}, xref = { other }}").unwrap();
        let renames = vec![
            ("procs".to_string(), "ng2020proc".to_string()),
            ("other".to_string(), "lee2019x".to_string()),
        ];
        assert_eq!(rename_references(&mut bib.entries, &renames), 2);
        assert_eq!(bib.entries[0].get("crossref"), Some("ng2020proc"));
        assert_eq!(bib.entries[0].get("xref"), Some("lee2019x"));
    }

    #[test]
    fn renames_citations_in_tex() {
        let renames = vec![
            ("tb".to_string(), "knuth1984texbook".to_string()),
            ("x".to_string(), "y".to_string()),
        ];
        let tex = r"As in \cite{tb}, see \citep[p.~3]{ x, other} and \textcite*[see][]{tb}. Not \ref{tb} or \emph{x}. Caf\'e \nocite{x}";
        let (out, n) = rename_citations(tex, &renames);
        assert_eq!(
            out,
            r"As in \cite{knuth1984texbook}, see \citep[p.~3]{ y, other} and \textcite*[see][]{knuth1984texbook}. Not \ref{tb} or \emph{x}. Caf\'e \nocite{y}"
        );
        assert_eq!(n, 4);
    }
}
