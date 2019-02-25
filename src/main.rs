mod input;
mod logger;
mod ser;

use failure::{Error, Fallible};
use grep_cli::{is_tty_stdout, StandardStream};
use structopt::clap::AppSettings;
use structopt::StructOpt;
use termcolor::ColorChoice;

use crate::input::Input;

/// json-view is a utility for viewing json files in the terminal.
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

    let mut input = input::read(&opts.input)?;
    if let Some(ptr) = &opts.pointer {
        match input {
            Input::File(file, _) => ser::project(opts.ser, ptr, file, stdout()),
            Input::Buffer(cursor) => ser::project(opts.ser, ptr, cursor, stdout()),
            Input::Stdin(stdin) => ser::project(opts.ser, ptr, stdin.lock(), stdout()),
        }
    } else {
        match input {
            Input::File(file, _) => ser::shorten(opts.ser, file, stdout()),
            _ => ser::shorten(opts.ser, input.to_buffer()?, stdout()),
        }
    }
}

fn stdout() -> StandardStream {
    let color_choice = if is_tty_stdout() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    grep_cli::stdout(color_choice)
}

fn fmt_error(err: &Error) -> String {
    let mut pretty = err.to_string();
    for cause in err.iter_causes() {
        pretty.push_str(&format!("\ncaused by: {}", cause));
    }
    pretty
}
