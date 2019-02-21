mod input;
mod logger;

use std::io;

use failure::{Error, Fallible, ResultExt};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(flatten)]
    input: input::Opts,
    #[structopt(flatten)]
    logger: logger::Opts,
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

    input::read(&opts.input, |rdr| {
        io::copy(rdr, &mut io::stdout().lock()).context("failed writing to stdout")?;
        Ok(())
    })
}

fn fmt_error(err: &Error) -> String {
    let mut pretty = err.to_string();
    for cause in err.iter_causes() {
        pretty.push_str(&format!("\ncaused by: {}", cause));
    }
    pretty
}
