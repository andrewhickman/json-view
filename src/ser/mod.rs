mod count;
mod exclude;
#[cfg(test)]
mod tests;

use std::io::{Read, Seek, SeekFrom, Write};

use failure::{Fallible, ResultExt};
use json::de::Deserializer;
use serde::ser::{Serialize, Serializer};
use serde_transcode::transcode;
use structopt::StructOpt;

#[derive(Copy, Clone, Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Opts {
    /// The maximum number of lines a json value can take up when printed.
    #[structopt(long, short = "L", default_value = "64")]
    max_length: u32,
    /// The maximum depth to which a json value should be printed.
    #[structopt(long, short = "D")]
    max_depth: Option<u32>,
}

impl Opts {
    fn is_identity(&self) -> bool {
        self.max_length == 0 && self.max_depth.is_none()
    }
}

pub fn project<R, W>(opts: Opts, ptr: &str, rdr: R, wtr: W) -> Fallible<()>
where
    R: Read,
    W: Write,
{
    let value: json::Value = json::from_reader(rdr).context("Failed to read json from input")?;
    if let Some(proj) = value.pointer(ptr) {
        if opts.is_identity() {
            Ok(json::to_writer_pretty(wtr, proj)?)
        } else {
            let excludes = count::count(opts, |ser| {
                Ok(proj.serialize(ser)?)
            })?;
            exclude::write(excludes, wtr, |ser| {
                Ok(proj.serialize(ser).context("Failed to write output")?)
            })
        }
    } else {
        log::warn!("No value found for json pointer {}", ptr);
        Ok(())
    }
}

pub fn shorten<R, W>(opts: Opts, mut rdr: R, wtr: W) -> Fallible<()>
where
    R: Read + Seek,
    W: Write,
{
    if opts.is_identity() {
        serialize(rdr, &mut json::Serializer::pretty(wtr))
    } else {
        let excludes = count::count(opts, |ser| {
            Ok(serialize(rdr.by_ref(), ser).context("Failed to read json from input")?)
        })?;
        rdr.seek(SeekFrom::Start(0))?;
        exclude::write(excludes, wtr, |ser| {
            Ok(serialize(rdr, ser).context("Failed to write output")?)
        })
    }
}

fn serialize<R, S>(rdr: R, ser: S) -> Fallible<()>
where
    R: Read,
    S: Serializer,
    S::Error: Send + Sync + 'static,
{
    let mut de = Deserializer::from_reader(rdr);
    transcode(&mut de, ser)?;
    Ok(())
}
