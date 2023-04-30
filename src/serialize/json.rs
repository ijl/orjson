// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// This is an adaptation of `src/value/ser.rs` from serde-json.

use crate::serialize::writer::WriteExt;
use serde::ser::{self, Impossible, Serialize};
use serde_json::error::{Error, Result};
use std::io;

macro_rules! reserve_minimum {
    ($writer:expr) => {
        $writer.reserve(64);
    };
}

pub struct Serializer<W, F = CompactFormatter> {
    writer: W,
    formatter: F,
}

impl<W> Serializer<W>
where
    W: io::Write + WriteExt,
{
    #[inline]
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, CompactFormatter)
    }
}

impl<W> Serializer<W, PrettyFormatter>
where
    W: io::Write + WriteExt,
{
    #[inline]
    pub fn pretty(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::new())
    }
}

impl<W, F> Serializer<W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    #[inline]
    pub fn with_formatter(writer: W, formatter: F) -> Self {
        Serializer { writer, formatter }
    }

    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
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

    #[cold]
    fn serialize_i8(self, value: i8) -> Result<()> {
        self.formatter
            .write_i8(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[cold]
    fn serialize_i16(self, value: i16) -> Result<()> {
        self.formatter
            .write_i16(&mut self.writer, value)
            .map_err(Error::io)
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

    #[cold]
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.formatter
            .write_u8(&mut self.writer, value)
            .map_err(Error::io)
    }

    #[cold]
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.formatter
            .write_u16(&mut self.writer, value)
            .map_err(Error::io)
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

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        format_escaped_str(&mut self.writer, &mut self.formatter, value).map_err(Error::io)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
        unreachable!();
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        self.formatter
            .write_null(&mut self.writer)
            .map_err(Error::io)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unreachable!();
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
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len == Some(0) {
            unsafe {
                reserve_minimum!(self.writer);
                self.writer.write_reserved_fragment(b"[]").unwrap();
            }
            Ok(Compound {
                ser: self,
                state: State::Empty,
            })
        } else {
            self.formatter
                .begin_array(&mut self.writer)
                .map_err(Error::io)?;
            Ok(Compound {
                ser: self,
                state: State::First,
            })
        }
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
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        if len == Some(0) {
            unsafe {
                reserve_minimum!(self.writer);
                self.writer.write_reserved_fragment(b"{}").unwrap();
            }

            Ok(Compound {
                ser: self,
                state: State::Empty,
            })
        } else {
            self.formatter
                .begin_object(&mut self.writer)
                .map_err(Error::io)?;
            Ok(Compound {
                ser: self,
                state: State::First,
            })
        }
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
pub enum State {
    Empty,
    First,
    Rest,
}

pub struct Compound<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
    state: State,
}

impl<'a, W, F> ser::SerializeSeq for Compound<'a, W, F>
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
            .map_err(Error::io)?;
        self.state = State::Rest;
        value.serialize(&mut *self.ser)?;
        self.ser
            .formatter
            .end_array_value(&mut self.ser.writer)
            .map_err(Error::io)
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => Ok(()),
            _ => self
                .ser
                .formatter
                .end_array(&mut self.ser.writer)
                .map_err(Error::io),
        }
    }
}

impl<'a, W, F> ser::SerializeTuple for Compound<'a, W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!();
    }

    fn end(self) -> Result<()> {
        unreachable!();
    }
}

impl<'a, W, F> ser::SerializeMap for Compound<'a, W, F>
where
    W: io::Write + WriteExt,
    F: Formatter,
{
    type Ok = ();
    type Error = Error;

    #[inline(always)]
    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.ser
            .formatter
            .begin_object_key(&mut self.ser.writer, self.state == State::First)
            .map_err(Error::io)?;
        key.serialize(MapKeySerializer { ser: self.ser })?;

        self.ser
            .formatter
            .end_object_key(&mut self.ser.writer)
            .map_err(Error::io)?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)
            .map_err(Error::io)?;
        value.serialize(&mut *self.ser)?;
        self.ser
            .formatter
            .end_object_value(&mut self.ser.writer)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser
            .formatter
            .begin_object_key(&mut self.ser.writer, self.state == State::First)
            .map_err(Error::io)?;
        self.state = State::Rest;

        key.serialize(MapKeySerializer { ser: self.ser })?;

        self.ser
            .formatter
            .end_object_key(&mut self.ser.writer)
            .map_err(Error::io)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)
            .map_err(Error::io)?;
        value.serialize(&mut *self.ser)?;
        self.ser
            .formatter
            .end_object_value(&mut self.ser.writer)
            .map_err(Error::io)
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => Ok(()),
            _ => self
                .ser
                .formatter
                .end_object(&mut self.ser.writer)
                .map_err(Error::io),
        }
    }
}

#[repr(transparent)]
struct MapKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ser::Serializer for MapKeySerializer<'a, W, F>
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

    #[inline]
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

pub enum CharEscape {
    /// An escaped quote `"`
    Quote,
    /// An escaped reverse solidus `\`
    ReverseSolidus,
    /// An escaped backspace character (usually escaped as `\b`)
    Backspace,
    /// An escaped form feed character (usually escaped as `\f`)
    FormFeed,
    /// An escaped line feed character (usually escaped as `\n`)
    LineFeed,
    /// An escaped carriage return character (usually escaped as `\r`)
    CarriageReturn,
    /// An escaped tab character (usually escaped as `\t`)
    Tab,
    /// An escaped ASCII plane control character (usually escaped as
    /// `\u00XX` where `XX` are two hex characters)
    AsciiControl(u8),
}

impl CharEscape {
    #[inline]
    fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
        match escape {
            self::BB => CharEscape::Backspace,
            self::TT => CharEscape::Tab,
            self::NN => CharEscape::LineFeed,
            self::FF => CharEscape::FormFeed,
            self::RR => CharEscape::CarriageReturn,
            self::QU => CharEscape::Quote,
            self::BS => CharEscape::ReverseSolidus,
            self::UU => CharEscape::AsciiControl(byte),
            _ => unreachable!(),
        }
    }
}

pub trait Formatter {
    #[inline]
    fn write_null<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            writer.write_reserved_fragment(b"null")
        }
    }

    #[inline]
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        let s = if value {
            b"true" as &[u8]
        } else {
            b"false" as &[u8]
        };
        reserve_minimum!(writer);
        unsafe { writer.write_reserved_fragment(s) }
    }

    #[inline]
    fn write_i8<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_i16<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_i32<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_i128<W>(&mut self, _writer: &mut W, _value: i128) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        unreachable!();
    }

    #[inline]
    fn write_u8<W>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_u16<W>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_u32<W>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_u64<W>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = itoap::write_to_ptr(writer.as_mut_buffer_ptr(), value);
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_u128<W>(&mut self, _writer: &mut W, _value: u128) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        unreachable!();
    }

    #[inline]
    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = ryu::raw::format32(value, writer.as_mut_buffer_ptr());
            writer.set_written(len);
        }
        Ok(())
    }

    #[inline]
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unsafe {
            reserve_minimum!(writer);
            let len = ryu::raw::format64(value, writer.as_mut_buffer_ptr());
            writer.set_written(len);
        }
        Ok(())
    }

    fn write_number_str<W>(&mut self, _writer: &mut W, _value: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        unreachable!();
    }

    #[inline]
    fn begin_string<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unreachable!();
    }

    #[inline]
    fn end_string<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unreachable!();
    }

    #[inline]
    fn write_string_fragment<W>(&mut self, _writer: &mut W, _fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unreachable!();
    }

    #[inline]
    fn write_char_escape<W>(&mut self, writer: &mut W, char_escape: CharEscape) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        use self::CharEscape::*;

        let s = match char_escape {
            Quote => b"\\\"",
            ReverseSolidus => b"\\\\",
            Backspace => b"\\b",
            FormFeed => b"\\f",
            LineFeed => b"\\n",
            CarriageReturn => b"\\r",
            Tab => b"\\t",
            AsciiControl(byte) => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                let bytes = &[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX_DIGITS[(byte >> 4) as usize],
                    HEX_DIGITS[(byte & 0xF) as usize],
                ];
                return unsafe { writer.write_reserved_fragment(bytes) };
            }
        };

        unsafe { writer.write_reserved_fragment(s) }
    }

    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        reserve_minimum!(writer);
        unsafe { writer.write_reserved_punctuation(b'[').unwrap() };
        Ok(())
    }

    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        reserve_minimum!(writer);
        unsafe { writer.write_reserved_punctuation(b']').unwrap() };
        Ok(())
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        if !first {
            unsafe {
                reserve_minimum!(writer);
                writer.write_reserved_punctuation(b',').unwrap()
            }
        }
        Ok(())
    }

    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        reserve_minimum!(writer);
        unsafe {
            writer.write_reserved_punctuation(b'{').unwrap();
        }
        Ok(())
    }

    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        reserve_minimum!(writer);
        unsafe {
            writer.write_reserved_punctuation(b'}').unwrap();
        }
        Ok(())
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        if !first {
            unsafe {
                reserve_minimum!(writer);
                writer.write_reserved_punctuation(b',').unwrap();
            }
        }
        Ok(())
    }

    #[inline]
    fn end_object_key<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        reserve_minimum!(writer);
        unsafe { writer.write_reserved_punctuation(b':') }
    }

    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        Ok(())
    }
}

pub struct CompactFormatter;

impl Formatter for CompactFormatter {}

pub struct PrettyFormatter {
    current_indent: usize,
    has_value: bool,
}

impl PrettyFormatter {
    pub fn new() -> Self {
        PrettyFormatter {
            current_indent: 0,
            has_value: false,
        }
    }
}

impl Default for PrettyFormatter {
    fn default() -> Self {
        PrettyFormatter::new()
    }
}

impl Formatter for PrettyFormatter {
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        self.current_indent += 1;
        self.has_value = false;
        reserve_minimum!(writer);
        unsafe { writer.write_reserved_punctuation(b'[') }
    }

    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        self.current_indent -= 1;
        let num_spaces = self.current_indent * 2;
        writer.reserve(num_spaces + 2);

        unsafe {
            if self.has_value {
                writer.write_reserved_punctuation(b'\n')?;
                writer.write_reserved_indent(num_spaces)?;
            }
            writer.write_reserved_punctuation(b']')
        }
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        let num_spaces = self.current_indent * 2;
        writer.reserve(num_spaces + 2);

        unsafe {
            writer.write_reserved_fragment(if first { b"\n" } else { b",\n" })?;
            writer.write_reserved_indent(num_spaces)?;
        };
        Ok(())
    }

    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        Ok(())
    }

    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        self.current_indent += 1;
        self.has_value = false;

        reserve_minimum!(writer);
        unsafe { writer.write_reserved_punctuation(b'{') }
    }

    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        self.current_indent -= 1;
        let num_spaces = self.current_indent * 2;
        writer.reserve(num_spaces + 2);

        unsafe {
            if self.has_value {
                writer.write_reserved_punctuation(b'\n')?;
                writer.write_reserved_indent(num_spaces)?;
            }

            writer.write_reserved_punctuation(b'}')
        }
    }

    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        let num_spaces = self.current_indent * 2;
        writer.reserve(num_spaces + 2);
        unsafe {
            writer.write_reserved_fragment(if first { b"\n" } else { b",\n" })?;
            writer.write_reserved_indent(num_spaces)?;
        }
        Ok(())
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        reserve_minimum!(writer);
        unsafe { writer.write_reserved_fragment(b": ").unwrap() };
        Ok(())
    }

    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.has_value = true;
        Ok(())
    }
}

fn format_escaped_str<W, F>(writer: &mut W, formatter: &mut F, value: &str) -> io::Result<()>
where
    W: ?Sized + io::Write + WriteExt,
    F: ?Sized + Formatter,
{
    let len = value.len();

    if len == 0 {
        reserve_minimum!(writer);
        return unsafe { writer.write_reserved_fragment(b"\"\"") };
    }

    unsafe {
        let mut escapes: u8 = __;
        let mut idx = 0;
        let as_bytes = value.as_bytes();
        while idx < len.saturating_sub(8) {
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 1) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 2) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 3) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 4) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 5) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 6) as usize);
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx + 7) as usize);
            if unlikely!(escapes != __) {
                return format_escaped_str_with_escapes(writer, formatter, as_bytes, idx);
            }
            idx += 8;
        }
        while idx < len {
            escapes |= *ESCAPE.get_unchecked(*as_bytes.get_unchecked(idx) as usize);
            if unlikely!(escapes != __) {
                return format_escaped_str_with_escapes(writer, formatter, as_bytes, idx);
            }
            idx += 1;
        }
    }

    writer.write_str(value)
}

fn format_escaped_str_with_escapes<W, F>(
    writer: &mut W,
    formatter: &mut F,
    value: &[u8],
    initial: usize,
) -> io::Result<()>
where
    W: ?Sized + io::Write + WriteExt,
    F: ?Sized + Formatter,
{
    writer.reserve((value.len() * 8) + 2);
    unsafe {
        writer.write_reserved_punctuation(b'"').unwrap();
        if initial > 0 {
            writer
                .write_reserved_fragment(value.get_unchecked(0..initial))
                .unwrap();
        }
        format_escaped_str_contents(writer, formatter, value.get_unchecked(initial..)).unwrap();
        writer.write_reserved_punctuation(b'"').unwrap();
    };
    Ok(())
}

fn format_escaped_str_contents<W, F>(
    writer: &mut W,
    formatter: &mut F,
    bytes: &[u8],
) -> io::Result<()>
where
    W: ?Sized + io::Write + WriteExt,
    F: ?Sized + Formatter,
{
    let len = bytes.len();
    let mut start = 0;
    let mut idx = 0;

    let mut escape: u8;
    loop {
        if idx < len.saturating_sub(4) {
            escape = 0;
            unsafe {
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx) as usize);
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx + 1) as usize);
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx + 2) as usize);
                escape |= *ESCAPE.get_unchecked(*bytes.get_unchecked(idx + 3) as usize);
            }
            if escape == 0 {
                idx += 4;
                continue;
            }
        }

        let byte = unsafe { *bytes.get_unchecked(idx) };
        escape = unsafe { *ESCAPE.get_unchecked(byte as usize) };
        if escape == 0 {
            idx += 1;
            if idx == len {
                break;
            } else {
                continue;
            }
        }

        if start < idx {
            unsafe {
                writer
                    .write_reserved_fragment(bytes.get_unchecked(start..idx))
                    .unwrap()
            };
        }

        let char_escape = CharEscape::from_escape_table(escape, byte);
        formatter.write_char_escape(writer, char_escape)?;

        idx += 1;
        start = idx;
        if idx == len {
            break;
        }
    }

    if start != len {
        unsafe {
            writer
                .write_reserved_fragment(bytes.get_unchecked(start..len))
                .unwrap()
        };
    }
    Ok(())
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
const ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write + WriteExt,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

#[inline]
pub fn to_writer_pretty<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write + WriteExt,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::pretty(writer);
    value.serialize(&mut ser)
}
