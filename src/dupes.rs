//! Finding the same work entered twice under different keys.

use crate::bib::Entry;
use crate::text::fold;

/// Two entries that look like the same work, by index, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct Pair {
    pub first: usize,
    pub second: usize,
    pub reason: &'static str,
}

fn doi(e: &Entry) -> Option<String> {
    let d = e.get("doi")?.trim().to_lowercase();
    // A DOI pasted as a link is still the same DOI.
    let d = d
        .rsplit_once("doi.org/")
        .map_or(d.as_str(), |(_, rest)| rest)
        .to_string();
    (!d.is_empty()).then_some(d)
}

/// Share of distinct words the two titles have in common (Jaccard index).
fn overlap(a: &str, b: &str) -> f64 {
    let a: std::collections::BTreeSet<&str> = a.split(' ').filter(|w| !w.is_empty()).collect();
    let b: std::collections::BTreeSet<&str> = b.split(' ').filter(|w| !w.is_empty()).collect();
    let union = a.union(&b).count();
    if union == 0 {
        0.0
    } else {
        a.intersection(&b).count() as f64 / union as f64
    }
}

/// Titles this close, with the same year, count as the same work: one
/// extra or changed word in a ten-word title still matches.
const NEAR: f64 = 0.8;
/// Short titles can be close by chance ("Introduction", "A Survey").
const MIN_WORDS: usize = 5;

/// Pairs with the same DOI, or the same (or nearly the same) title and year
/// once case, accents, braces and punctuation are ignored.
pub fn find(entries: &[Entry]) -> Vec<Pair> {
    let dois: Vec<Option<String>> = entries.iter().map(doi).collect();
    let titles: Vec<String> = entries
        .iter()
        .map(|e| fold(e.get("title").unwrap_or("")))
        .collect();
    let years: Vec<&str> = entries
        .iter()
        .map(|e| e.get("year").unwrap_or("").trim())
        .collect();
    let mut out = Vec::new();
    for j in 0..entries.len() {
        for i in 0..j {
            let reason = if dois[i].is_some() && dois[i] == dois[j] {
                "same DOI"
            } else if !titles[i].is_empty() && titles[i] == titles[j] && years[i] == years[j] {
                "same title and year"
            } else if years[i] == years[j]
                && titles[i].split(' ').count() >= MIN_WORDS
                && titles[j].split(' ').count() >= MIN_WORDS
                && overlap(&titles[i], &titles[j]) >= NEAR
            {
                "nearly the same title, same year"
            } else {
                continue;
            };
            out.push(Pair {
                first: i,
                second: j,
                reason,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bib::parse;

    fn pairs(src: &str) -> Vec<(usize, usize, &'static str)> {
        find(&parse(src).unwrap().entries)
            .into_iter()
            .map(|p| (p.first, p.second, p.reason))
            .collect()
    }

    #[test]
    fn same_doi_even_as_a_link() {
        let p = pairs(
            "@misc{a, doi={10.1145/362384.362685}}\n@misc{b, doi={https://doi.org/10.1145/362384.362685}}",
        );
        assert_eq!(p, [(0, 1, "same DOI")]);
    }

    #[test]
    fn same_title_however_it_is_typed() {
        let p = pairs(
            r#"@article{a, title={{G}\"{o}del's Proof}, year=1958}
@book{b, title={GÖDEL'S PROOF.}, year={1958}}
@book{c, title={Gödel's Proof}, year={2001}}"#,
        );
        assert_eq!(p, [(0, 1, "same title and year")]);
    }

    #[test]
    fn nearly_the_same_title() {
        let p = pairs(
            "@misc{a, title={Attention Is All You Need for Translation}, year=2017}\n@misc{b, title={Attention Is All You Need for Machine Translation}, year=2017}",
        );
        assert_eq!(p, [(0, 1, "nearly the same title, same year")]);
    }

    #[test]
    fn short_or_different_titles_dont_match() {
        assert!(
            pairs(
                "@misc{a, title={A Survey}, year=2020}\n@misc{b, title={A New Survey}, year=2020}"
            )
            .is_empty()
        );
        assert!(pairs("@misc{a, title={Deep Learning for Graphs and Text}, year=2020}\n@misc{b, title={Shallow Models for Images and Sound}, year=2020}").is_empty());
        assert!(pairs("@misc{a}\n@misc{b}").is_empty());
    }
}
