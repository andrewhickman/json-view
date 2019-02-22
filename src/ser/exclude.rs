use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::io::Write;
use std::ops::Range;

use failure::Fallible;
use json::ser::PrettyFormatter;
use serde::{de, ser};
use serde_transcode::transcode;

pub fn to_writer<'de, D, W>(excludes: ExcludeSet, de: D, wtr: W) -> Fallible<()>
where
    D: de::Deserializer<'de>,
    W: Write,
{
    let ser = Serializer {
        excludes,
        exclude_depth: 0,
        position: 0,
        ser: &mut json::Serializer::pretty(wtr),
    };
    transcode(de, ser)?;
    Ok(())
}

#[derive(Debug)]
pub struct ExcludeSet {
    indices: BTreeSet<Exclude>,
}

struct Serializer<'a, W> {
    excludes: ExcludeSet,
    exclude_depth: u32,
    position: u32,
    ser: &'a mut json::Serializer<W, PrettyFormatter<'static>>,
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

impl<'a, W> ser::Serializer for Serializer<'a, W>
where
    W: Write,
{
    type Ok = <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::Ok;
    type Error = <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::Error;
    type SerializeSeq =
        <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::SerializeSeq;
    type SerializeTuple =
        <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::SerializeTuple;
    type SerializeTupleStruct =
        <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant =
        <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::SerializeTupleVariant;
    type SerializeMap =
        <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::SerializeMap;
    type SerializeStruct =
        <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::SerializeStruct;
    type SerializeStructVariant =
        <&'a mut json::Serializer<W, PrettyFormatter<'static>> as ser::Serializer>::SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_i8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_i16(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_i32(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_u32(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_f32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_str(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_bytes(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_none()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.ser.serialize_some(value)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_unit()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_unit_struct(name)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.ser
            .serialize_unit_variant(name, variant_index, variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.ser.serialize_newtype_struct(name, value)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.ser
            .serialize_newtype_variant(name, variant_index, variant, value)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.ser.serialize_seq(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.ser.serialize_tuple(len)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.ser.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.ser
            .serialize_tuple_variant(name, variant_index, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.ser.serialize_map(len)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.ser.serialize_struct(name, len)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.ser
            .serialize_struct_variant(name, variant_index, variant, len)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_i128(v)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.ser.serialize_u128(v)
    }

    fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: ser::Serialize,
    {
        self.ser.collect_seq(iter)
    }

    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        K: ser::Serialize,
        V: ser::Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        self.ser.collect_map(iter)
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Display,
    {
        self.ser.collect_str(value)
    }

    fn is_human_readable(&self) -> bool {
        self.ser.is_human_readable()
    }
}
