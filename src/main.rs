use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use clap::{ArgAction, Parser};

use bindery::scanner::CodeScanner;

#[derive(Parser, Debug)]
#[command(
    name = "bindery",
    about = "Concatenate project source files for review/analysis"
)]
struct Cli {
    /// Include hidden files and directories (default: ignore)
    #[arg(short = 'a', long = "all", action = ArgAction::SetTrue)]
    include_hidden: bool,

    /// Strip comments from source before concatenation (default: keep)
    #[arg(short = 'n', long = "no-comments", action = ArgAction::SetTrue)]
    no_comments: bool,

    /// Output file path. If omitted, write to stdout. The output file is excluded from scanning.
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Paths to scan. Defaults to current directory if none provided.
    #[arg()]
    paths: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut paths = if cli.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.paths
    };

    // Normalize to absolute paths so we can make consistent relative strings later
    for p in &mut paths {
        if let Ok(abs) = p.canonicalize() {
            *p = abs;
        }
    }

    let output_path = cli.output.as_ref().map(|p| p.clone());

    let excluded = vec![String::from(".git")];

    let scanner = CodeScanner::new(
        paths,
        excluded,
        cli.include_hidden,
        cli.no_comments,
        output_path.clone(),
    );

    let output = scanner.concatenate()?;

    match output_path {
        Some(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, output)?;
            eprintln!("Written to {}", path.display());
        }
        None => {
            let mut stdout = io::BufWriter::new(io::stdout());
            stdout.write_all(output.as_bytes())?;
        }
    }

    Ok(())
}
