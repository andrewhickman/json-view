use std::fs::{self, File};
use std::io::{self, stdin, BufRead, BufReader};
use std::path::{Path, PathBuf};

use failure::{err_msg, Fallible, ResultExt};
use grep_cli::is_readable_stdin;
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
    #[structopt(flatten)]
    data: DataOpts,
}

#[derive(Debug, StructOpt)]
pub struct DataOpts {
    #[structopt(
        name = "DATA",
        long = "data",
        short = "d",
        help = "Data file to read from",
        global = true,
        parse(from_os_str)
    )]
    file: Option<PathBuf>,
    #[structopt(long, env = "JSON_VIEW_DATA_DIR", global = true, parse(from_os_str))]
    dir: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct Start {
    #[structopt(long, short)]
    force: bool,
    #[structopt(long)]
    append: bool,
}

impl Start {
    pub fn run(&self, opts: &Opts) -> Fallible<()> {
        let dir = opts.data.dir()?;
        log::trace!("Creating data directory {}", dir.display());
        fs::create_dir_all(&dir)
            .context(format!("failed to create directory {}", dir.display()))?;

        if !is_readable_stdin() {
            return Err(err_msg("could not read from stdin"));
        }

        let path = opts.data.file(dir);
        log::debug!("Creating data file {}", path.display());
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .append(self.append)
            .create_new(!self.force)
            .open(&path)
        {
            Ok(file) => Ok(file),
            Err(ref err) if err.kind() == io::ErrorKind::AlreadyExists => {
                Err(err_msg("will not overwrite file without --force flag"))
            }
            Err(err) => Err(err.into()),
        }
        .context(format!("failed to create file {}", path.display()))?;

        io::copy(&mut stdin().lock(), &mut file)
            .context(format!("failed to write to file {}", path.display()))?;
        Ok(())
    }
}

pub fn read<F, R>(opts: &Opts, mut f: F) -> Fallible<R>
where
    F: FnMut(&mut dyn BufRead) -> Fallible<R>,
{
    if let Some(path) = &opts.input {
        log::debug!("Reading from input file {}.", path.display());
        f(&mut BufReader::new(open(path)?))
    } else if is_readable_stdin() {
        log::debug!("Reading from stdin.");
        f(&mut stdin().lock())
    } else {
        let dir = opts.data.dir()?;
        let path = opts.data.file(dir);
        log::debug!("Reading from data file {}.", path.display());
        f(&mut BufReader::new(open(&path)?))
    }
}

fn open(path: &Path) -> Fallible<File> {
    Ok(File::open(path).context(format!("failed to open file {}", path.display()))?)
}

impl DataOpts {
    fn file(&self, dir: PathBuf) -> PathBuf {
        dir.join::<&Path>(match &self.file {
            Some(file) => file.as_ref(),
            None => "data".as_ref(),
        })
        .with_extension("json")
    }

    fn dir(&self) -> Fallible<PathBuf> {
        if let Some(dir) = &self.dir {
            Ok(dir.to_owned())
        } else if let Some(dir) = dirs::data_dir() {
            Ok(dir.join("json-view"))
        } else {
            Err(err_msg(
                "data directory option not set and failed to find standard data directory",
            ))
        }
    }
}
