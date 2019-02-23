mod count;
mod exclude;

use std::io::{Read, Seek, SeekFrom, Write};

use failure::Fallible;
use json::de::Deserializer;
use structopt::StructOpt;

#[derive(Copy, Clone, Debug, StructOpt)]
pub struct Opts {
    /// The maximum number of lines a json value can take up when printed.
    #[structopt(long, short = "L", default_value = "64")]
    max_length: u32,
    /// The maximum depth to which a json value should be printed.
    #[structopt(long, short = "D")]
    max_depth: Option<u32>,
}

pub fn to_writer<R, W>(opts: Opts, mut rdr: R, wtr: W) -> Fallible<()>
where
    R: Read + Seek,
    W: Write,
{
    let excludes = count::count(opts, &mut Deserializer::from_reader(rdr.by_ref()))?;
    rdr.seek(SeekFrom::Start(0))?;
    exclude::to_writer(excludes, &mut Deserializer::from_reader(rdr), wtr)
}
