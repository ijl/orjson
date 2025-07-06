// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// This is an adaptation of `src/value/ser.rs` from serde-json.

use crate::serialize::writer::formatter::{CompactFormatter, Formatter, PrettyFormatter};
use crate::serialize::writer::WriteExt;
use serde::ser::{self, Impossible, Serialize};
use serde_json::error::{Error, Result};
use std::io;

pub(crate) struct Serializer<W, F = CompactFormatter> {
    writer: W,
    formatter: F,
    ensure_ascii: bool,
}

impl<W> Serializer<W>
where
    W: io::Write + WriteExt,
{
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter, false)
    }

    #[inline]
    pub fn with_ascii(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter, true)
    }
}

impl<W> Serializer<W, PrettyFormatter>
where
    W: io::Write + WriteExt,
{
    #[inline]
    pub fn pretty(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::new(), false)
    }

    #[inline]
    pub fn pretty_ascii(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::new(), true)
    }
}

impl<W, F> Serializer<W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    #[inline]
    pub fn with_formatter(writer: W, formatter: F, ensure_ascii: bool) -> Self {
        Serializer {
            writer,
            formatter,
            ensure_ascii,
        }
    }
}

impl<'a, W, F> ser::Serializer for &'a mut Serializer<W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a, W, F>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Compound<'a, W, F>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        self.formatter
            .write_bool(&mut self.writer, value)
            .map_err(Error::io)
    }

    fn serialize_i8(self, _value: i8) -> Result<()> {
        unreachable!();
    }

    fn serialize_i16(self, _value: i16) -> Result<()> {
        unreachable!();
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        self.formatter
            .write_i32(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        self.formatter
            .write_i64(&mut self.writer, value)
            .map_err(Error::io)
    }

    fn serialize_i128(self, _value: i128) -> Result<()> {
        unreachable!();
    }

    fn serialize_u8(self, _value: u8) -> Result<()> {
        unreachable!();
    }

    fn serialize_u16(self, _value: u16) -> Result<()> {
        unreachable!();
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        self.formatter
            .write_u32(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.formatter
            .write_u64(&mut self.writer, value)
            .map_err(Error::io)
    }

    fn serialize_u128(self, _value: u128) -> Result<()> {
        unreachable!();
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<()> {
        if unlikely!(value.is_infinite() || value.is_nan()) {
            self.serialize_unit()
        } else {
            self.formatter
                .write_f32(&mut self.writer, value)
                .map_err(Error::io)
        }
    }
    #[inline]
    fn serialize_f64(self, value: f64) -> Result<()> {
        if unlikely!(value.is_infinite() || value.is_nan()) {
            self.serialize_unit()
        } else {
            self.formatter
                .write_f64(&mut self.writer, value)
                .map_err(Error::io)
        }
    }

    fn serialize_char(self, _value: char) -> Result<()> {
        unreachable!();
    }

    #[inline(always)]
    fn serialize_str(self, value: &str) -> Result<()> {
        if self.ensure_ascii {
            format_escaped_str_ascii(&mut self.writer, value);
        } else {
            format_escaped_str(&mut self.writer, value);
        }
        Ok(())
    }

    #[inline(always)]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.writer.reserve(value.len() + 32);
        unsafe {
            self.writer.write_reserved_fragment(value).unwrap();
        }
        Ok(())
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.formatter
            .write_null(&mut self.writer)
            .map_err(Error::io)
    }

    #[inline(always)]
    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        debug_assert!(name.len() <= 36);
        reserve_minimum!(self.writer);
        unsafe {
            self.writer.write_reserved_punctuation(b'"').unwrap();
            self.writer
                .write_reserved_fragment(name.as_bytes())
                .unwrap();
            self.writer.write_reserved_punctuation(b'"').unwrap();
        }
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        unreachable!();
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!();
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!();
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline(always)]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.formatter
            .begin_array(&mut self.writer)
            .map_err(Error::io)?;
        Ok(Compound {
            ser: self,
            state: State::First,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unreachable!();
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unreachable!();
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unreachable!();
    }

    #[inline(always)]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.formatter
            .begin_object(&mut self.writer)
            .map_err(Error::io)?;
        Ok(Compound {
            ser: self,
            state: State::First,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        unreachable!();
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unreachable!();
    }
}

#[derive(Eq, PartialEq)]
pub(crate) enum State {
    First,
    Rest,
}

pub(crate) struct Compound<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
    state: State,
}

impl<W, F> ser::SerializeSeq for Compound<'_, W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser
            .formatter
            .begin_array_value(&mut self.ser.writer, self.state == State::First)
            .unwrap();
        self.state = State::Rest;
        value.serialize(&mut *self.ser)?;
        self.ser
            .formatter
            .end_array_value(&mut self.ser.writer)
            .map_err(Error::io)
            .unwrap();
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.ser.formatter.end_array(&mut self.ser.writer).unwrap();
        Ok(())
    }
}

impl<W, F> ser::SerializeMap for Compound<'_, W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_entry<K, V>(&mut self, _key: &K, _value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        unreachable!()
    }

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser
            .formatter
            .begin_object_key(&mut self.ser.writer, self.state == State::First)
            .unwrap();
        self.state = State::Rest;

        key.serialize(MapKeySerializer { ser: self.ser })?;

        self.ser
            .formatter
            .end_object_key(&mut self.ser.writer)
            .unwrap();
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)
            .unwrap();
        value.serialize(&mut *self.ser)?;
        self.ser
            .formatter
            .end_object_value(&mut self.ser.writer)
            .unwrap();
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.ser.formatter.end_object(&mut self.ser.writer).unwrap();
        Ok(())
    }
}

#[repr(transparent)]
struct MapKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<W, F> ser::Serializer for MapKeySerializer<'_, W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    #[inline(always)]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.ser.serialize_str(value)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        unreachable!();
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!();
    }
    fn serialize_bool(self, _value: bool) -> Result<()> {
        unreachable!();
    }

    fn serialize_i8(self, _value: i8) -> Result<()> {
        unreachable!();
    }

    fn serialize_i16(self, _value: i16) -> Result<()> {
        unreachable!();
    }

    fn serialize_i32(self, _value: i32) -> Result<()> {
        unreachable!();
    }

    fn serialize_i64(self, _value: i64) -> Result<()> {
        unreachable!();
    }

    fn serialize_i128(self, _value: i128) -> Result<()> {
        unreachable!();
    }

    fn serialize_u8(self, _value: u8) -> Result<()> {
        unreachable!();
    }

    fn serialize_u16(self, _value: u16) -> Result<()> {
        unreachable!();
    }

    fn serialize_u32(self, _value: u32) -> Result<()> {
        unreachable!();
    }

    fn serialize_u64(self, _value: u64) -> Result<()> {
        unreachable!();
    }

    fn serialize_u128(self, _value: u128) -> Result<()> {
        unreachable!();
    }

    fn serialize_f32(self, _value: f32) -> Result<()> {
        unreachable!();
    }

    fn serialize_f64(self, _value: f64) -> Result<()> {
        unreachable!();
    }

    fn serialize_char(self, _value: char) -> Result<()> {
        unreachable!();
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
        unreachable!();
    }

    fn serialize_unit(self) -> Result<()> {
        unreachable!();
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unreachable!();
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!();
    }

    fn serialize_none(self) -> Result<()> {
        unreachable!();
    }

    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!();
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unreachable!();
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        unreachable!();
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unreachable!();
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unreachable!();
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unreachable!();
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        unreachable!();
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unreachable!();
    }
}

macro_rules! reserve_str {
    ($writer:expr, $value:expr) => {
        $writer.reserve($value.len() * 8 + 32);
    };
}

#[cfg(all(target_arch = "x86_64", feature = "avx512"))]
type StrFormatter = unsafe fn(*mut u8, *const u8, usize) -> usize;

#[cfg(all(target_arch = "x86_64", feature = "avx512"))]
static mut STR_FORMATTER_FN: StrFormatter =
    crate::serialize::writer::str::format_escaped_str_impl_sse2_128;

pub(crate) fn set_str_formatter_fn() {
    unsafe {
        #[cfg(all(target_arch = "x86_64", feature = "avx512"))]
        if std::is_x86_feature_detected!("avx512vl") {
            STR_FORMATTER_FN = crate::serialize::writer::str::format_escaped_str_impl_512vl;
        }
    }
}

#[cfg(all(target_arch = "x86_64", not(feature = "avx512")))]
#[inline(always)]
fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + io::Write + WriteExt,
{
    unsafe {
        reserve_str!(writer, value);

        let written = crate::serialize::writer::str::format_escaped_str_impl_sse2_128(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.set_written(written);
    }
}

#[cfg(all(target_arch = "x86_64", feature = "avx512"))]
#[inline(always)]
fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + io::Write + WriteExt,
{
    unsafe {
        reserve_str!(writer, value);

        let written = STR_FORMATTER_FN(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.set_written(written);
    }
}

#[cfg(all(
    not(target_arch = "x86_64"),
    not(feature = "avx512"),
    feature = "generic_simd"
))]
#[inline(always)]
fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + io::Write + WriteExt,
{
    unsafe {
        reserve_str!(writer, value);

        let written = crate::serialize::writer::str::format_escaped_str_impl_generic_128(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.set_written(written);
    }
}

#[cfg(all(not(target_arch = "x86_64"), not(feature = "generic_simd")))]
#[inline(always)]
fn format_escaped_str<W>(writer: &mut W, value: &str)
where
    W: ?Sized + io::Write + WriteExt,
{
    unsafe {
        reserve_str!(writer, value);

        let written = crate::serialize::writer::str::format_escaped_str_scalar(
            writer.as_mut_buffer_ptr(),
            value.as_bytes().as_ptr(),
            value.len(),
        );

        writer.set_written(written);
    }
}

fn format_escaped_str_ascii<W>(writer: &mut W, value: &str)
where
    W: ?Sized + io::Write + WriteExt,
{
    // Worst case: every char becomes \uXXXX plus quotes.
    writer.reserve(value.len() * 6 + 2);

    unsafe {
        writer.write_reserved_punctuation(b'"').unwrap();

        for c in value.chars() {
            match c {
                '"' => writer.write_reserved_fragment(b"\\\"").unwrap(),
                '\\' => writer.write_reserved_fragment(b"\\\\").unwrap(),
                '\u{08}' => writer.write_reserved_fragment(b"\\b").unwrap(),
                '\u{0C}' => writer.write_reserved_fragment(b"\\f").unwrap(),
                '\n' => writer.write_reserved_fragment(b"\\n").unwrap(),
                '\r' => writer.write_reserved_fragment(b"\\r").unwrap(),
                '\t' => writer.write_reserved_fragment(b"\\t").unwrap(),
                c if c.is_ascii() => {
                    // Write ASCII character directly.
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    writer.write_reserved_fragment(s.as_bytes()).unwrap();
                }
                c => {
                    // Write as \uXXXX.
                    let code = c as u32;
                    let buf = match code {
                        0x0000..=0xFFFF => {
                            // Basic Multilingual Plane
                            let s = format!("\\u{:04x}", code);
                            debug_assert_eq!(s.len(), 6);
                            s
                        }
                        _ => {
                            // Encode surrogate pairs.
                            let code = code - 0x1_0000;
                            let high = 0xD800 | ((code >> 10) & 0x3FF);
                            let low = 0xDC00 | (code & 0x3FF);
                            format!("\\u{:04x}\\u{:04x}", high, low)
                        }
                    };
                    writer.write_reserved_fragment(buf.as_bytes()).unwrap();
                }
            }
        }

        writer.write_reserved_punctuation(b'"').unwrap();
    }
}

#[inline]
pub(crate) fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write + WriteExt,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

#[inline]
pub(crate) fn to_writer_ascii<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write + WriteExt,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::with_ascii(writer);
    value.serialize(&mut ser)
}

#[inline]
pub(crate) fn to_writer_pretty<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write + WriteExt,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::pretty(writer);
    value.serialize(&mut ser)
}

#[inline]
pub(crate) fn to_writer_pretty_ascii<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write + WriteExt,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::pretty_ascii(writer);
    value.serialize(&mut ser)
}
