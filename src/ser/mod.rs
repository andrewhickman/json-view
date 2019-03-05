mod count;
mod exclude;
#[cfg(test)]
mod tests;

use std::io::{Read, Seek, SeekFrom, Write};

use failure::{Error, Fallible};
use json::de::Deserializer;
use serde::ser::{Serialize, Serializer};
use serde_transcode::transcode;
use structopt::StructOpt;

#[derive(Copy, Clone, Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Opts {
    /// The maximum number of lines a JSON value can take up when printed.
    #[structopt(long, short = "L", raw(validator = "non_zero"))]
    max_length: Option<u32>,
    /// The maximum depth to which a JSON value should be printed.
    #[structopt(long, short = "D")]
    max_depth: Option<u32>,
}

fn non_zero(arg: String) -> Result<(), String> {
    if let Ok(0) = arg.parse() {
        Err("maximum length must be greater than 0".to_owned())
    } else {
        Ok(())
    }
}

impl Opts {
    fn is_identity(&self) -> bool {
        self.max_length.is_none() && self.max_depth.is_none()
    }
}

pub fn project<R, W>(opts: Opts, ptr: &str, rdr: R, wtr: W) -> Fallible<()>
where
    R: Read,
    W: Write,
{
    let value: json::Value = json::from_reader(rdr).map_err(wrap_json_err)?;
    if let Some(proj) = value.pointer(ptr) {
        if opts.is_identity() {
            Ok(json::to_writer_pretty(wtr, proj).map_err(wrap_json_err)?)
        } else {
            let excludes = count::count(opts, |ser| proj.serialize(ser).map_err(wrap_json_err))?;
            exclude::write(excludes, wtr, |ser| {
                proj.serialize(ser).map_err(wrap_json_err)
            })
        }
    } else {
        log::warn!("No value found for JSON pointer `{}`.", ptr);
        Ok(())
    }
}

pub fn shorten<R, W>(opts: Opts, mut rdr: R, wtr: W) -> Fallible<()>
where
    R: Read + Seek,
    W: Write,
{
    if opts.is_identity() {
        identity(rdr, wtr)
    } else {
        let excludes = count::count(opts, |ser| serialize(rdr.by_ref(), ser))?;
        rdr.seek(SeekFrom::Start(0))?;
        exclude::write(excludes, wtr, |ser| serialize(rdr, ser))
    }
}

pub fn identity<R, W>(rdr: R, wtr: W) -> Fallible<()>
where
    R: Read,
    W: Write,
{
    serialize(rdr, &mut json::Serializer::pretty(wtr))
}

fn serialize<R, S>(rdr: R, ser: S) -> Fallible<()>
where
    R: Read,
    S: Serializer<Ok = (), Error = json::Error>,
{
    let mut de = Deserializer::from_reader(rdr);
    transcode(&mut de, ser).map_err(wrap_json_err)
}

fn wrap_json_err(e: json::Error) -> Error {
    // TODO: preserve error through transcode and add context
    e.into()
}
