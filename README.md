# bibtidy

Cleans up a BibTeX file before it goes into a paper. It rewrites the file in one consistent style, points out
entries that will print badly, finds the same paper entered twice, and renames citation keys to one pattern,
updating your `.tex` files to match.

## Install

```sh
cargo install --path .
```

Needs Rust 1.85 or newer. No network.

## Use

`examples/refs.bib` is a small, deliberately untidy bibliography of well-known papers.

```
$ bibtidy check examples/refs.bib
dijkstra: pages "147-148" should use -- for a range
Lamport1978: doi "https://doi.org/10.1145/359545.359563" is a link; the field wants just the 10.xxxx/... part
bert: pages "4171-4186" should use -- for a range
bert: title has capitals most styles will lower-case; write {BERT}
4 problems in 10 entries.
```

`check` exits with 1 when it finds something, so it can sit in CI next to the paper.

```
$ bibtidy dupes examples/refs.bib
lamport78 and Lamport1978: same DOI
    Time, Clocks, and the Ordering of Events in a Distributed System
```

Two entries count as the same work when they share a DOI (even if one is written as a link), or have the same
year and the same title once case, accents, braces and punctuation are ignored. Titles of five words or more
also match when about four in five of their words are shared, which catches a dropped or extra word.

```
$ bibtidy keys examples/refs.bib
dijkstra -> dijkstra1968go
lamport78 -> lamport1978timea
Lamport1978 -> lamport1978timeb
codd -> codd1970relational
...
```

Keys are surname, year and the first word of the title that isn't "a", "the", "on" and so on. Entries that
would clash get a letter on the end. `--write` saves the new keys, and `--tex paper.tex` also rewrites them
inside `\cite`, `\citep`, `\textcite`, `\nocite` and any other command with "cite" in its name.

```
$ bibtidy fmt examples/refs.bib --fix
Fixed 4 fields.
@article{dijkstra,
  author  = {Dijkstra, Edsger W.},
  title   = {Go To Statement Considered Harmful},
  journal = {Communications of the ACM},
  volume  = {11},
  number  = {3},
  pages   = {147--148},
  year    = {1968},
  doi     = {10.1145/362929.362947},
}
...
```

`fmt` prints to stdout unless you pass `--in-place`.

- Fields come out in a fixed order (author, editor, title, booktitle, journal, then volume, number, pages,
  publisher and so on, year and month last of the standard ones). Anything else keeps its place after them.
  `--keep-order` turns this off.
- `--sort key` or `--sort year` reorders the entries.
- `--fix` changes single-hyphen page ranges to `--`, strips `https://doi.org/` from DOIs, and turns line breaks
  inside values into spaces.

## What check looks at

- Fields the standard BibTeX styles need for each entry type, for example journal and year for `@article`.
  biblatex's `date` counts as a year, and `@online`, `@report`, `@thesis`, `@software` and `@dataset` are known.
- Years that aren't four digits, page ranges with one hyphen, DOIs given as links or not starting with `10.`,
  and URLs with spaces.
- Keys used twice, ignoring case as BibTeX does.
- Words in titles with capitals after the first letter that aren't in braces. Most styles lower-case titles,
  so `BERT` prints as "Bert" unless written `{BERT}`. `--no-caps` skips this.

## Limits

- `@string` abbreviations and month names are expanded when writing, so `journal = cacm` comes back as
  `journal = {Communications of the ACM}` and `month = jul` as `month = {July}`. The output means the same, but
  the abbreviations are gone.
- Comments and other text between entries are dropped by `fmt`.
- Accents are understood for the common LaTeX commands (`\"`, `\'`, `` \` ``, `\^`, `\~`, `\c`, `\v`, `\H`, `\r`
  and letters like `\ss` and `\o`). Rarer ones are passed through as the bare letter when comparing.
