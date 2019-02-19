#[derive(Copy, Clone, Debug, StructOpt)]
pub struct Opts {
    #[structopt(long = "debug", help = "Enables debug logging", global = true)]
    debug: bool,
    #[structopt(long = "trace", help = "Enables trace logging", global = true)]
    trace: bool,
}