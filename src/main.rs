mod input;
mod logger;
mod ser;

use failure::{Error, Fallible, ResultExt};
use grep_cli::{is_tty_stdout, StandardStream};
use structopt::clap::AppSettings;
use structopt::StructOpt;
use termcolor::ColorChoice;

use crate::input::Input;

/// json-view is a utility for viewing json files in the terminal.
#[derive(Debug, StructOpt)]
#[structopt(raw(global_settings = "&[AppSettings::UnifiedHelpMessage]"))]
pub struct Opts {
    #[structopt(flatten)]
    input: input::Opts,
    #[structopt(flatten)]
    logger: logger::Opts,
    #[structopt(flatten)]
    ser: ser::Opts,
    #[structopt(subcommand)]
    cmd: Option<Command>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "start", about = "Initialize data file from stdin")]
    Start(input::Start),
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
        };
    }

    let input = input::read(&opts.input)?;
    match input {
        Input::File(file) => ser::to_writer(opts.ser, file, stdout()),
        Input::Memory(cursor) => ser::to_writer(opts.ser, cursor, stdout()),
    }
    .context("Failed to write to stdout")?;
    Ok(())
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
