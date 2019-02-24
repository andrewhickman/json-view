mod count;
mod exclude;
#[cfg(test)]
mod tests;

use std::io::{Read, Seek, SeekFrom, Write};

use failure::{Fallible, ResultExt};
use json::de::Deserializer;
use structopt::StructOpt;
use serde_transcode::transcode;

#[derive(Copy, Clone, Debug, StructOpt)]
pub struct Opts {
    /// The maximum number of lines a json value can take up when printed.
    #[structopt(long, short = "L", default_value = "64")]
    max_length: u32,
    /// The maximum depth to which a json value should be printed.
    #[structopt(long, short = "D")]
    max_depth: Option<u32>,
}

pub fn shorten<R, W>(opts: Opts, mut rdr: R, wtr: W) -> Fallible<()>
where
    R: Read + Seek,
    W: Write,
{
    let excludes = count::count(opts, |ser| {
        let mut de = Deserializer::from_reader(rdr.by_ref());
        transcode(&mut de, ser).context("Failed to read json from input")?;
        Ok(())
    })?;

    rdr.seek(SeekFrom::Start(0))?;

    exclude::write(excludes, wtr, |ser| {
        let mut de = Deserializer::from_reader(rdr);
        transcode(&mut de, ser).context(format!("Failed to write"))?;
        Ok(())
    })
}
