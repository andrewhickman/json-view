use std::fs::{self, File};
use std::io::{self, stdin, BufReader, Cursor, Read};
use std::path::{Path, PathBuf};

use failure::{err_msg, Fallible, ResultExt};
use grep_cli::is_readable_stdin;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// A path to the input file, or '-' to read from stdin
    #[structopt(
        name = "INPUT",
        help = "Input file, or '-' to read from stdin",
        conflicts_with = "data",
        parse(from_os_str)
    )]
    input: Option<PathBuf>,
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
    #[structopt(name = "DIR", long = "data-dir", global = true, parse(from_os_str))]
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
}

pub enum Input {
    File(BufReader<File>),
    Memory(Cursor<String>),
}

impl Start {
    pub fn run(&self, opts: &Opts) -> Fallible<()> {
        let dir = opts.data.dir()?;
        log::trace!("Creating data directory {}", dir.display());
        fs::create_dir_all(&dir)
            .context(format!("Failed to create directory {}", dir.display()))?;

        if !is_readable_stdin() {
            return Err(err_msg("Stdin not readable"));
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
                Err(err_msg("Will not overwrite file without --force flag"))
            }
            Err(err) => Err(err.into()),
        }
        .context(format!("Failed to create file {}", path.display()))?;

        io::copy(&mut stdin().lock(), &mut file)
            .context(format!("Failed to write to file {}", path.display()))?;
        log::info!("Created data file {}", path.display());
        Ok(())
    }
}

pub fn read(opts: &Opts) -> Fallible<Input> {
    if let Some(path) = &opts.input {
        log::debug!("Reading from input file {}.", path.display());
        Input::file(path)
    } else if is_readable_stdin() {
        log::debug!("Reading from stdin.");
        Input::stdin()
    } else {
        let dir = opts.data.dir()?;
        let path = opts.data.file(dir);
        log::debug!("Reading from data file {}.", path.display());
        Input::file(path)
    }
}

impl Input {
    fn file(path: impl AsRef<Path>) -> Fallible<Self> {
        Ok(Input::File(BufReader::new(
            File::open(path.as_ref())
                .context(format!("Failed to open file {}", path.as_ref().display()))?,
        )))
    }

    fn stdin() -> Fallible<Self> {
        let mut buf = String::new();
        stdin().lock().read_to_string(&mut buf)?;
        Ok(Input::Memory(Cursor::new(buf)))
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
            Err(err_msg(
                "Data directory option not set and failed to find standard data directory",
            ))
        }
    }
}
