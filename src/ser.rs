use std::io::Write;
use std::ops::Range;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use serde::ser;
use structopt::StructOpt;

#[derive(Copy, Clone, Debug, StructOpt)]
pub struct Opts {
    /// The maximum number of lines a json value can take up when printed.
    #[structopt(long, default_value = 64)]
    max_length: u32,
    /// The maximum depth to which a json value should be printed.
    #[structopt(long)]
    max_depth: Option<u32>,
}

struct Serializer<W> {
    writer: W,
    indices: BinaryHeap<Index>,
}

struct Index {
    depth: u32,
    length: usize,
    range: Range<usize>,
}

impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Index {
    fn cmp(&self, other: &Self) -> Ordering {
        self.depth.cmp(&other.depth)
            .then(self.length.cmp(&other.length))

    }
}

impl<W> ser::Serializer for Serializer<W>
where
    W: Write
{

}
