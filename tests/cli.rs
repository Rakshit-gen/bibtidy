use std::path::PathBuf;
use std::process::{Command, Output};

const SAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/refs.bib");

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bibtidy")).args(args).output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A copy of the sample that a test can change.
fn copy(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::copy(SAMPLE, &path).unwrap();
    path
}

#[test]
fn check_finds_the_planted_problems() {
    let out = run(&["check", SAMPLE]);
    assert_eq!(out.status.code(), Some(1));
    let s = text(&out.stdout);
    assert!(s.contains("dijkstra: pages \"147-148\" should use -- for a range"), "{s}");
    assert!(s.contains("bert: title has capitals most styles will lower-case; write {BERT}"), "{s}");
    assert_eq!(s.lines().count(), 4, "{s}");
}

#[test]
fn fix_then_check_leaves_only_the_title() {
    let path = copy("fixed.bib");
    let p = path.to_str().unwrap();
    assert!(run(&["fmt", p, "--fix", "--in-place"]).status.success());
    let s = text(&run(&["check", p]).stdout);
    assert_eq!(s.trim(), "bert: title has capitals most styles will lower-case; write {BERT}");
}

#[test]
fn fmt_output_is_stable() {
    let once = text(&run(&["fmt", SAMPLE]).stdout);
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("once.bib");
    std::fs::write(&path, &once).unwrap();
    let twice = text(&run(&["fmt", path.to_str().unwrap()]).stdout);
    assert_eq!(once, twice);
    assert!(once.contains("  journal = {Communications of the ACM},"), "{once}");
}

#[test]
fn dupes_finds_the_lamport_pair() {
    let out = run(&["dupes", SAMPLE]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stdout).starts_with("lamport78 and Lamport1978: same DOI"));
}

#[test]
fn keys_rename_the_bib_and_the_tex() {
    let path = copy("keys.bib");
    let tex = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("paper.tex");
    std::fs::write(&tex, r"As \citet{codd} showed \cite[p.~2]{texbook, hoare}.").unwrap();
    let out = run(&["keys", path.to_str().unwrap(), "--write", "--tex", tex.to_str().unwrap()]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(&tex).unwrap(),
        r"As \citet{codd1970relational} showed \cite[p.~2]{knuth1984texbook, hoare1962quicksort}."
    );
    assert!(std::fs::read_to_string(&path).unwrap().contains("@article{codd1970relational,"));
}

#[test]
fn a_broken_file_names_the_line() {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("broken.bib");
    std::fs::write(&path, "@misc{a, title={fine}}\n@misc{b,\n  title = {never closed\n").unwrap();
    let out = run(&["check", path.to_str().unwrap()]);
    assert!(!out.status.success());
    let err = text(&out.stderr);
    assert!(err.contains("isn't valid BibTeX") && err.contains("line 3"), "{err}");
}
