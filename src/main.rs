use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use bibtidy::bib::{Bib, parse};
use bibtidy::tidy::{self, SortBy};
use bibtidy::{check, dupes, keys, write};

#[derive(Parser)]
#[command(version, about = "Format, check and de-duplicate BibTeX files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Rewrite a .bib file in one consistent style
    Fmt {
        /// The .bib file to read
        file: PathBuf,
        /// Overwrite the file instead of printing the result
        #[arg(short, long)]
        in_place: bool,
        /// Leave fields in the order they were written
        #[arg(long)]
        keep_order: bool,
        /// Sort the entries; by default they stay in file order
        #[arg(long, value_enum)]
        sort: Option<Sort>,
        /// Also fix page ranges, DOI links and stray line breaks in values
        #[arg(long)]
        fix: bool,
    },
    /// Rename citation keys to lastname, year, first title word
    Keys {
        /// The .bib file to read
        file: PathBuf,
        /// Save the renamed entries back to the file instead of only listing changes
        #[arg(short, long)]
        write: bool,
        /// Also update \cite commands in these .tex files (needs --write)
        #[arg(long, requires = "write")]
        tex: Vec<PathBuf>,
    },
    /// List missing fields, odd values and duplicate keys; exits 1 if any
    Check {
        /// The .bib file to read
        file: PathBuf,
        /// Don't warn about capitals in titles
        #[arg(long)]
        no_caps: bool,
    },
    /// List entries that look like the same work; exits 1 if any
    Dupes {
        /// The .bib file to read
        file: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Sort {
    Key,
    Year,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Fmt {
            file,
            in_place,
            keep_order,
            sort,
            fix,
        } => {
            let mut bib = load(&file)?;
            if !keep_order {
                bib.entries.iter_mut().for_each(tidy::order_fields);
            }
            if fix {
                let n: usize = bib.entries.iter_mut().map(tidy::fix).sum();
                eprintln!("Fixed {n} fields.");
            }
            match sort {
                Some(Sort::Key) => tidy::sort_entries(&mut bib.entries, SortBy::Key),
                Some(Sort::Year) => tidy::sort_entries(&mut bib.entries, SortBy::Year),
                None => {}
            }
            let out = write::bib(&bib);
            if in_place {
                std::fs::write(&file, out)
                    .with_context(|| format!("couldn't write {}", file.display()))?;
            } else {
                print!("{out}");
            }
        }
        Command::Keys { file, write, tex } => {
            let mut bib = load(&file)?;
            let new = keys::assign(&bib.entries);
            let mut renames = Vec::new();
            for (e, k) in bib.entries.iter_mut().zip(new) {
                if e.key != k {
                    println!("{} -> {k}", e.key);
                    renames.push((std::mem::replace(&mut e.key, k.clone()), k));
                }
            }
            let changed = renames.len();
            if changed == 0 {
                eprintln!("Every key already follows the pattern.");
            } else if write {
                std::fs::write(&file, write::bib(&bib))
                    .with_context(|| format!("couldn't write {}", file.display()))?;
                eprintln!("Renamed {changed} keys in {}.", file.display());
                for path in &tex {
                    let text = std::fs::read_to_string(path)
                        .with_context(|| format!("couldn't read {}", path.display()))?;
                    let (text, n) = keys::rename_citations(&text, &renames);
                    std::fs::write(path, text)
                        .with_context(|| format!("couldn't write {}", path.display()))?;
                    eprintln!("Updated {n} citations in {}.", path.display());
                }
                if tex.is_empty() {
                    eprintln!(
                        "Citations in your .tex files still use the old keys; pass them with --tex to update them."
                    );
                }
            }
        }
        Command::Check { file, no_caps } => {
            let bib = load(&file)?;
            let problems = check::all(&bib.entries, !no_caps);
            for p in &problems {
                println!("{}: {}", p.key, p.message);
            }
            if problems.is_empty() {
                eprintln!("{} entries, no problems found.", bib.entries.len());
            } else {
                let n = problems.len();
                eprintln!(
                    "{n} problem{} in {} entries.",
                    if n == 1 { "" } else { "s" },
                    bib.entries.len()
                );
                std::process::exit(1);
            }
        }
        Command::Dupes { file } => {
            let bib = load(&file)?;
            let pairs = dupes::find(&bib.entries);
            for p in &pairs {
                let (a, b) = (&bib.entries[p.first], &bib.entries[p.second]);
                println!("{} and {}: {}", a.key, b.key, p.reason);
                println!(
                    "    {}",
                    bibtidy::text::plain(a.get("title").unwrap_or("(no title)"))
                );
            }
            if pairs.is_empty() {
                eprintln!("No duplicates among {} entries.", bib.entries.len());
            } else {
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

fn load(path: &Path) -> Result<Bib> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("couldn't read {}", path.display()))?;
    parse(&text).with_context(|| format!("{} isn't valid BibTeX", path.display()))
}
