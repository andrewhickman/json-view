use std::io::{self, prelude::*};

use console::{style, Color};
use log::{Log, SetLoggerError};
use structopt::StructOpt;

pub fn init(opts: &Opts) -> Result<(), SetLoggerError> {
    log::set_max_level(opts.level_filter());
    log::set_logger(&Logger)
}

#[derive(Debug)]
struct Logger;

#[derive(Debug)]
struct Padded<W> {
    writer: W,
    lvl: Option<log::Level>,
}

#[derive(Copy, Clone, Debug, StructOpt)]
pub struct Opts {
    #[structopt(long = "debug", help = "Enables debug logging", global = true)]
    debug: bool,
    #[structopt(long = "trace", help = "Enables trace logging", global = true)]
    trace: bool,
}

impl Opts {
    fn level_filter(&self) -> log::LevelFilter {
        if self.trace {
            log::LevelFilter::Trace
        } else if self.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        }
    }
}

impl Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(&record.metadata()) {
            Padded {
                writer: io::stderr().lock(),
                lvl: Some(record.level()),
            }
            .write_fmt(*record.args())
            .unwrap_or_else(|err| {
                if err.kind() != io::ErrorKind::BrokenPipe {
                    panic!("error writing to stderr: {}", err);
                }
            });
        }
    }

    fn flush(&self) {}
}

impl<W: Write> Write for Padded<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for line in io::Cursor::new(buf).lines() {
            if let Some(lvl) = self.lvl.take() {
                let color = match lvl {
                    log::Level::Trace => Color::White,
                    log::Level::Debug => Color::Cyan,
                    log::Level::Info => Color::Magenta,
                    log::Level::Warn => Color::Yellow,
                    log::Level::Error => Color::Red,
                };

                writeln!(self.writer, "{:>8}: {}", style(lvl).fg(color), line?)?;
            } else {
                writeln!(self.writer, "{:>8}  {}", "", line?)?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
