mod input;
mod io;
mod logger;
mod ser;

use std::io::Write;

use failure::{Error, Fallible};
use structopt::clap::AppSettings;
use structopt::StructOpt;

use crate::io::Input;

/// json-view is a utility for viewing JSON files in the terminal.
#[derive(Debug, StructOpt)]
#[structopt(raw(global_setting = "AppSettings::UnifiedHelpMessage"))]
#[structopt(raw(global_setting = "AppSettings::VersionlessSubcommands"))]
pub struct Opts {
    #[structopt(flatten)]
    input: input::Opts,
    #[structopt(flatten)]
    logger: logger::Opts,
    #[structopt(flatten)]
    ser: ser::Opts,
    /// A pointer to select a value to output.
    #[structopt(long, short)]
    pointer: Option<String>,
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "start", about = "Initialize a data file from stdin")]
    Start(input::Start),
    #[structopt(name = "clean", about = "Remove all data files")]
    Clean(input::Clean),
}

fn main() {
    let opts = Opts::from_args();
    logger::init(opts.logger).unwrap();
    log::trace!("Options: {:#?}.", opts);

    if let Err(err) = run(&opts) {
        log::error!("{}", fmt_error(&err));
    }
}

fn run(opts: &Opts) -> Fallible<()> {
    if let Some(cmd) = &opts.cmd {
        return match cmd {
            Command::Start(start) => start.run(&opts.input),
            Command::Clean(clean) => clean.run(&opts.input),
        };
    }

    let mut stdout = io::stdout();
    let mut input = input::read(&opts.input)?;
    let result = if let Some(ptr) = &opts.pointer {
        match input {
            Input::File(file) => ser::project(opts.ser, ptr, file, &mut stdout),
            Input::Buffer(cursor) => ser::project(opts.ser, ptr, cursor, &mut stdout),
            Input::Stdin(stdin) => ser::project(opts.ser, ptr, stdin.lock(), &mut stdout),
        }
    } else {
        match input {
            Input::File(file) => ser::shorten(opts.ser, file, &mut stdout),
            _ => ser::shorten(opts.ser, input.to_buffer()?, &mut stdout),
        }
    };

    result.and(stdout.flush().map_err(Into::into))
}

fn fmt_error(err: &Error) -> String {
    let mut pretty = err.to_string();
    for cause in err.iter_causes() {
        pretty.push_str(&format!("\ncaused by: {}", cause));
    }
    pretty
}
