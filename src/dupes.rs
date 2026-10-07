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
    let d = d.rsplit_once("doi.org/").map_or(d.as_str(), |(_, rest)| rest).to_string();
    (!d.is_empty()).then_some(d)
}

/// Share of distinct words the two titles have in common (Jaccard index).
fn overlap(a: &str, b: &str) -> f64 {
    let a: std::collections::BTreeSet<&str> = a.split(' ').filter(|w| !w.is_empty()).collect();
    let b: std::collections::BTreeSet<&str> = b.split(' ').filter(|w| !w.is_empty()).collect();
    let union = a.union(&b).count();
    if union == 0 { 0.0 } else { a.intersection(&b).count() as f64 / union as f64 }
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
    let titles: Vec<String> = entries.iter().map(|e| fold(e.get("title").unwrap_or(""))).collect();
    let years: Vec<&str> = entries.iter().map(|e| e.get("year").unwrap_or("").trim()).collect();
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
            out.push(Pair { first: i, second: j, reason });
        }
    }
    out
}
