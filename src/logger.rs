use std::io::{self, prelude::*};

use log::{Log, SetLoggerError};
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn init(opts: Opts) -> Result<(), SetLoggerError> {
    log::set_max_level(opts.level_filter());
    log::set_boxed_logger(Box::new(Logger::new()))
}

struct Logger {
    writer: StandardStream,
}

#[derive(Copy, Clone, Debug, StructOpt)]
pub struct Opts {
    #[structopt(long = "debug", help = "Enables debug logging", global = true)]
    debug: bool,
    #[structopt(long = "trace", help = "Enables trace logging", global = true)]
    trace: bool,
}

impl Opts {
    fn level_filter(self) -> log::LevelFilter {
        if self.trace {
            log::LevelFilter::Trace
        } else if self.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        }
    }
}

impl Logger {
    fn new() -> Self {
        let color_choice = if atty::is(atty::Stream::Stderr) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };

        Logger {
            writer: StandardStream::stderr(color_choice),
        }
    }

    fn write(&self, lvl: log::Level, msg: impl AsRef<str>) -> io::Result<()> {
        const PAD: usize = 8;

        let color = match lvl {
            log::Level::Trace => Color::White,
            log::Level::Debug => Color::Cyan,
            log::Level::Info => Color::Magenta,
            log::Level::Warn => Color::Yellow,
            log::Level::Error => Color::Red,
        };

        let mut writer = self.writer.lock();
        let mut lines = msg.as_ref().lines();

        if let Some(first) = lines.next() {
            writer.set_color(ColorSpec::new().set_fg(Some(color)))?;
            write!(writer, "{:>pad$}: ", lvl, pad = PAD)?;
            writer.reset()?;
            writeln!(writer, "{}", first)?;
        }
        for line in lines {
            writeln!(writer, "{:>pad$}  {}", line, "", pad = PAD)?;
        }

        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(&record.metadata()) {
            self.write(record.level(), &record.args().to_string())
                .unwrap_or_else(|err| {
                    if err.kind() != io::ErrorKind::BrokenPipe {
                        panic!("error writing to stderr: {}", err);
                    }
                });
        }
    }

    fn flush(&self) {}
}
