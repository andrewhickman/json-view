use std::borrow::Cow;
use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom, Stdin, StdinLock, Write};
use std::path::PathBuf;

use clipboard::{ClipboardContext, ClipboardProvider};
use failure::{err_msg, Fail, Fallible, ResultExt};
use grep_cli::{is_tty_stdout, StandardStream};
use termcolor::ColorChoice;

pub struct ReadWrapper<R> {
    rdr: R,
    label: Cow<'static, str>,
}

impl<R> Read for ReadWrapper<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.rdr
            .read(buf)
            .context(format!("Failed to read from {}", self.label))
            .map_err(wrap_fail)
    }

    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.rdr
            .read_to_string(buf)
            .context(format!("Failed to read from {}", self.label))
            .map_err(wrap_fail)
    }
}

impl<R> Seek for ReadWrapper<R>
where
    R: Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.rdr
            .seek(pos)
            .context(format!("Failed to seek in {}", self.label))
            .map_err(wrap_fail)
    }
}

impl ReadWrapper<Stdin> {
    pub fn lock(&self) -> ReadWrapper<StdinLock> {
        ReadWrapper {
            rdr: self.rdr.lock(),
            label: Cow::Borrowed("stdin"),
        }
    }
}

pub struct WriteWrapper<W> {
    wtr: W,
    label: Cow<'static, str>,
}

impl<W> Write for WriteWrapper<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.wtr
            .write(buf)
            .context(format!("Failed to write to {}", self.label.clone()))
            .map_err(wrap_fail)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.wtr
            .flush()
            .context(format!("Failed to flush {}", self.label.clone()))
            .map_err(wrap_fail)
    }
}

pub enum Input {
    File(ReadWrapper<BufReader<File>>),
    Stdin(ReadWrapper<Stdin>),
    Buffer(Cursor<String>),
}

impl Input {
    pub fn file(path: impl Into<PathBuf>) -> Fallible<Self> {
        let path = path.into();
        let label = format!("file `{}`", path.display());
        Ok(Input::File(ReadWrapper {
            rdr: BufReader::new(
                File::open(&path).context(format!("Failed to open {}", label.clone()))?,
            ),
            label: Cow::Owned(label),
        }))
    }

    pub fn stdin() -> Fallible<Self> {
        Ok(Input::Stdin(stdin()))
    }

    pub fn clipboard() -> Fallible<Self> {
        Ok(Input::Buffer(Cursor::new(read_clipboard()?)))
    }

    pub fn to_buffer(&mut self) -> Fallible<&mut Cursor<String>> {
        let mut buf = String::new();
        match self {
            Input::File(file) => file.read_to_string(&mut buf)?,
            Input::Stdin(stdin) => stdin.read_to_string(&mut buf)?,
            Input::Buffer(cursor) => return Ok(cursor),
        };

        *self = Input::Buffer(Cursor::new(buf));
        match self {
            Input::Buffer(cursor) => Ok(cursor),
            _ => unreachable!(),
        }
    }
}

pub fn stdout() -> WriteWrapper<StandardStream> {
    let color_choice = if is_tty_stdout() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    WriteWrapper {
        wtr: grep_cli::stdout(color_choice),
        label: Cow::Borrowed("stdout"),
    }
}

pub fn stdin() -> ReadWrapper<Stdin> {
    ReadWrapper {
        rdr: io::stdin(),
        label: Cow::Borrowed("stdin"),
    }
}

pub fn read_clipboard() -> Fallible<String> {
    let mut clip =
        wrap_std_err(ClipboardContext::new()).context("Failed to initialize clipboard")?;
    Ok(wrap_std_err(clip.get_contents()).context("Failed to read from clipboard")?)
}

fn wrap_std_err<T>(res: Result<T, Box<dyn std::error::Error>>) -> Fallible<T> {
    res.map_err(|e| err_msg(e.to_string()))
}

fn wrap_fail(e: impl Fail) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.compat())
}
