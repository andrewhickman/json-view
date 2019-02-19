use structopt::StructOpt;

mod logger;

#[derive(StructOpt)]
pub struct Opts {
    #[structopt(flatten)]
    logger: logger::Opts,
}

fn main() {
    let opts = Opts::from_args();
    logger::init(opts.logger).unwrap();

    log::error!("hello\nworld");
    log::warn!("hello\nworld");
    log::info!("hello\nworld");
    log::debug!("hello\nworld");
    log::trace!("hello\nworld");
}
