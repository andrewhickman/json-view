use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::io::Write;
use std::ops::Range;

use failure::{Fallible, Error};
use json::ser::PrettyFormatter;
use serde::{de, ser};
use serde_transcode::transcode;

pub fn to_writer<'de, D, W>(excludes: ExcludeSet, de: D, wtr: W) -> Fallible<()>
where
    D: de::Deserializer<'de>,
    W: Write,
{
    unimplemented!()
}

#[derive(Debug)]
pub struct ExcludeSet {
    indices: BTreeSet<Exclude>,
}

impl ExcludeSet {
    pub fn new() -> Self {
        ExcludeSet {
            indices: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, range: Range<u32>) {
        self.indices.insert(Exclude {
            index: range.start,
            bound: Bound::Start,
        });
        self.indices.insert(Exclude {
            index: range.end,
            bound: Bound::End,
        });
    }
}

#[derive(Debug)]
struct Exclude {
    index: u32,
    bound: Bound,
}

#[derive(Debug)]
enum Bound {
    Start,
    End,
}

impl PartialEq for Exclude {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for Exclude {}

impl PartialOrd for Exclude {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Exclude {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}
