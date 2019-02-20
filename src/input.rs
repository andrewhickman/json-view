use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::path::{Path, PathBuf};

use failure::{Fallible, ResultExt, err_msg};
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
        let file = opts.data.file();
        let path = dir.join(file).with_extension("json");
        log::debug!("Reading from data file {}.", path.display());
        f(&mut BufReader::new(open(&path)?))
    }
}

fn open(path: &Path) -> Fallible<File> {
    Ok(File::open(path).context(format!("failed to open {}", path.display()))?)
}

impl DataOpts {
    fn file(&self) -> &Path {
        match &self.file {
            Some(file) => file.as_ref(),
            None => "data".as_ref()
        }
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

pub fn is_readable_stdin() -> bool {
    #[cfg(unix)]
    fn imp() -> bool {
        use std::os::unix::fs::FileTypeExt;
        use same_file::Handle;

        let ft = match Handle::stdin().and_then(|h| h.as_file().metadata()) {
            Err(_) => return false,
            Ok(md) => md.file_type(),
        };
        ft.is_file() || ft.is_fifo()
    }

    #[cfg(windows)]
    fn imp() -> bool {
        use winapi_util as winutil;

        winutil::file::typ(winutil::HandleRef::stdin())
            .map(|t| t.is_disk() || t.is_pipe())
            .unwrap_or(false)
    }

    !atty::is(atty::Stream::Stdin) && imp()
}