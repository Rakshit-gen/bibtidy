use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use bibtidy::bib::{Bib, parse};
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
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Fmt { file, in_place } => {
            let bib = load(&file)?;
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
