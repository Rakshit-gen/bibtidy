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

/// Pairs with the same DOI, or the same title and year once case, accents,
/// braces and punctuation are ignored.
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
            } else {
                continue;
            };
            out.push(Pair { first: i, second: j, reason });
        }
    }
    out
}
