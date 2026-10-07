use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};

use bibtidy::bib::{Bib, parse};
use bibtidy::tidy::{self, SortBy};
use bibtidy::write;

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
    }
    Ok(())
}

fn load(path: &Path) -> Result<Bib> {
    let text = std::fs::read_to_string(path).with_context(|| format!("couldn't read {}", path.display()))?;
    parse(&text).with_context(|| format!("{} isn't valid BibTeX", path.display()))
}
