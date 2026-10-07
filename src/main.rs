use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use bibtidy::bib::{Bib, parse};
use bibtidy::tidy::{self, SortBy};
use bibtidy::{keys, write};

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
    },
    /// Rename citation keys to lastname, year, first title word
    Keys {
        /// The .bib file to read
        file: PathBuf,
        /// Save the renamed entries back to the file instead of only listing changes
        #[arg(short, long)]
        write: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Sort {
    Key,
    Year,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Fmt { file, in_place, keep_order, sort } => {
            let mut bib = load(&file)?;
            if !keep_order {
                bib.entries.iter_mut().for_each(tidy::order_fields);
            }
            match sort {
                Some(Sort::Key) => tidy::sort_entries(&mut bib.entries, SortBy::Key),
                Some(Sort::Year) => tidy::sort_entries(&mut bib.entries, SortBy::Year),
                None => {}
            }
            let out = write::bib(&bib);
            if in_place {
                std::fs::write(&file, out).with_context(|| format!("couldn't write {}", file.display()))?;
            } else {
                print!("{out}");
            }
        }
        Command::Keys { file, write } => {
            let mut bib = load(&file)?;
            let new = keys::assign(&bib.entries);
            let mut changed = 0;
            for (e, k) in bib.entries.iter_mut().zip(new) {
                if e.key != k {
                    println!("{} -> {k}", e.key);
                    e.key = k;
                    changed += 1;
                }
            }
            if changed == 0 {
                eprintln!("Every key already follows the pattern.");
            } else if write {
                std::fs::write(&file, write::bib(&bib)).with_context(|| format!("couldn't write {}", file.display()))?;
                eprintln!("Renamed {changed} keys in {}. Citations in your .tex files still use the old ones.", file.display());
            }
        }
    }
    Ok(())
}

fn load(path: &Path) -> Result<Bib> {
    let text = std::fs::read_to_string(path).with_context(|| format!("couldn't read {}", path.display()))?;
    parse(&text).with_context(|| format!("{} isn't valid BibTeX", path.display()))
}
