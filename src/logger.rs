use std::fmt::Write;
use std::fmt;

use console::{Color, style};
use log::{Log, SetLoggerError};
use structopt::StructOpt;

pub fn init(opts: &Opts) -> Result<(), SetLoggerError> {
    log::set_max_level(opts.level_filter());
    log::set_logger(&Logger)
}

#[derive(Debug)]
struct Logger;

#[derive(Debug)]
struct Message<'a> {
    lvl: log::Level, 
    args: fmt::Arguments<'a>,
}

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
            eprint!("{}", Message {
                lvl: record.level(), 
                args: *record.args(),
            });
        }
    }

    fn flush(&self) {}
}

impl fmt::Display for Message<'_> {
    fn fmt(&self, writer: &mut fmt::Formatter) -> fmt::Result {
        fmt::write(&mut Padded {
            writer,
            lvl: Some(self.lvl)
        }, self.args)
    }
}

impl<W: Write> fmt::Write for Padded<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for line in s.lines() {
            if let Some(lvl) = self.lvl.take() {
                let color = match lvl {
                    log::Level::Trace => Color::White,
                    log::Level::Debug => Color::Cyan,
                    log::Level::Info => Color::Magenta,
                    log::Level::Warn => Color::Yellow,
                    log::Level::Error => Color::Red,
                };

                writeln!(self.writer, "{:>8}: {}", style(lvl).fg(color), line)?;
            } else {
                writeln!(self.writer, "{:>8}  {}", "", line)?;
            }
        }
        Ok(())
    }
}
