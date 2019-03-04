use std::fs;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};

use failure::{ensure, format_err, Fallible, ResultExt};
use grep_cli::is_readable_stdin;
use structopt::StructOpt;

use crate::io::{read_clipboard, stdin, Input};
use crate::ser::identity;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// Input file to read from
    #[structopt(
        name = "INPUT",
        conflicts_with = "clipboard",
        conflicts_with = "data",
        parse(from_os_str)
    )]
    input: Option<PathBuf>,
    /// Read from clipboard
    #[structopt(long, short, conflicts_with = "data")]
    clipboard: bool,
    #[structopt(flatten)]
    data: DataOpts,
}

#[derive(Debug, StructOpt)]
pub struct DataOpts {
    /// The name of a file in the application's data directory to use as input.
    #[structopt(
        name = "data",
        long,
        short,
        help = "Data file to use",
        global = true,
        parse(from_os_str)
    )]
    file: Option<PathBuf>,
    /// The path the application should use for data.
    #[structopt(long = "data-dir", global = true, parse(from_os_str))]
    dir: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct Start {
    /// Overwrite the file if it already exists
    #[structopt(long, short)]
    force: bool,
    /// Append to the file if it already exists
    #[structopt(long, short, conflicts_with = "force")]
    append: bool,
    /// Read from clipboard
    #[structopt(long, short)]
    clipboard: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Clean {
    #[structopt(long)]
    dry_run: bool,
}

impl Start {
    pub fn run(&self, opts: &Opts) -> Fallible<()> {
        let dir = opts.data.dir()?;
        log::trace!("Creating data directory `{}`.", dir.display());
        fs::create_dir_all(&dir)
            .context(format!("Failed to create directory `{}`", dir.display()))?;

        if !self.clipboard {
            ensure!(is_readable_stdin(), "Stdin not readable");
        }

        let path = opts.data.file(dir);
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .append(self.append)
            .truncate(!self.append)
            .create_new(!self.force)
            .open(&path)
        {
            Ok(file) => Ok(file),
            Err(ref err) if err.kind() == io::ErrorKind::AlreadyExists => {
                Err(format_err!("Will not overwrite file without --force flag"))
            }
            Err(err) => Err(err.into()),
        }
        .context(format!("Failed to create file `{}`", path.display()))?;

        if self.clipboard {
            log::debug!("Initializing data file from clipboard.");
            let buf = read_clipboard()?;
            identity(Cursor::new(buf), &mut file)
        } else {
            log::debug!("Initializing data file from stdin.");
            identity(&mut stdin().lock(), &mut file)
        }
        .context(format!("Failed to initialize file `{}`", path.display()))?;

        log::info!("Created data file `{}`.", path.display());
        Ok(())
    }
}

impl Clean {
    pub fn run(&self, opts: &Opts) -> Fallible<()> {
        let dir = opts.data.dir()?;
        let entries = dir
            .read_dir()
            .context(format!("Failed to read directory `{}`", dir.display()))?;
        for entry in entries {
            let path = entry?.path();
            if !path.is_dir() {
                log::info!("Removing file `{}`.", path.display());
                if !self.dry_run {
                    fs::remove_file(&path)
                        .context(format!("Failed to remove file `{}`", path.display()))?;
                }
            } else {
                log::warn!("Ignoring unexpected directory `{}`.", path.display());
            }
        }
        Ok(())
    }
}

pub fn read(opts: &Opts) -> Fallible<Input> {
    if let Some(path) = &opts.input {
        log::debug!("Reading from input file `{}`.", path.display());
        Input::file(path)
    } else if opts.clipboard {
        log::debug!("Reading from clipboard.");
        Input::clipboard()
    } else if is_readable_stdin() {
        log::debug!("Reading from stdin.");
        Input::stdin()
    } else {
        let dir = opts.data.dir()?;
        let path = opts.data.file(dir);
        log::debug!("Reading from data file `{}`.", path.display());
        Input::file(path)
    }
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
            Err(format_err!(
                "Data directory option not set and failed to find standard data directory",
            ))
        }
    }
}
