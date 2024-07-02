// SPDX-License-Identifier: (Apache-2.0 OR MIT)
// This is an adaptation of `src/value/ser.rs` from serde-json.

use crate::serialize::writer::WriteExt;
use std::io;

macro_rules! debug_assert_has_capacity {
    ($writer:expr) => {
        debug_assert!($writer.has_capacity(4))
    };
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

    fn write_i8<W>(&mut self, _writer: &mut W, _value: i8) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unreachable!();
    }

    fn write_i16<W>(&mut self, _writer: &mut W, _value: i16) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unreachable!();
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

    fn write_i128<W>(&mut self, _writer: &mut W, _value: i128) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        unreachable!();
    }

    fn write_u8<W>(&mut self, _writer: &mut W, _value: u8) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unreachable!();
    }

    fn write_u16<W>(&mut self, _writer: &mut W, _value: u16) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        unreachable!();
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
        debug_assert_has_capacity!(writer);
        unsafe { writer.write_reserved_punctuation(b']').unwrap() };
        Ok(())
    }

    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write + WriteExt,
    {
        debug_assert_has_capacity!(writer);
        if !first {
            unsafe { writer.write_reserved_punctuation(b',').unwrap() }
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
        debug_assert_has_capacity!(writer);
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
        debug_assert_has_capacity!(writer);
        if !first {
            unsafe {
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
        debug_assert_has_capacity!(writer);
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
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        PrettyFormatter {
            current_indent: 0,
            has_value: false,
        }
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
        reserve_pretty!(writer, num_spaces);

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
        reserve_pretty!(writer, num_spaces);

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
        reserve_pretty!(writer, num_spaces);

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
        reserve_pretty!(writer, num_spaces);
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
