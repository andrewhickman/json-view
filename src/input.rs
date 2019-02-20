use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::path::{Path, PathBuf};

use failure::{Fallible, ResultExt};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(
        name = "INPUT",
        help = "Input file, or '-' to read from stdin",
        conflicts_with = "DATA",
        parse(from_os_str)
    )]
    input: Option<PathBuf>,
}

pub fn read<F, R>(opts: &Opts, mut f: F) -> Fallible<R>
where
    F: FnMut(&mut dyn BufRead) -> Fallible<R>,
{
    if let Some(path) = &opts.input {
        f(&mut BufReader::new(open(path)?))
    } else {
        f(&mut stdin().lock())
    }
}

fn open(path: &Path) -> Fallible<File> {
    Ok(File::open(path).context(format!("failed to open {}", path.display()))?)
}
