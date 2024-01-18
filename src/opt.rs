// SPDX-License-Identifier: (Apache-2.0 OR MIT)

pub type Opt = u32;

pub const INDENT_2: Opt = 1;
pub const NAIVE_UTC: Opt = 1 << 1;
pub const NON_STR_KEYS: Opt = 1 << 2;
pub const OMIT_MICROSECONDS: Opt = 1 << 3;
pub const SERIALIZE_NUMPY: Opt = 1 << 4;
pub const SORT_KEYS: Opt = 1 << 5;
pub const STRICT_INTEGER: Opt = 1 << 6;
pub const UTC_Z: Opt = 1 << 7;
pub const PASSTHROUGH_SUBCLASS: Opt = 1 << 8;
pub const PASSTHROUGH_DATETIME: Opt = 1 << 9;
pub const APPEND_NEWLINE: Opt = 1 << 10;
pub const PASSTHROUGH_DATACLASS: Opt = 1 << 11;

// deprecated
pub const SERIALIZE_DATACLASS: Opt = 0;
pub const SERIALIZE_UUID: Opt = 0;

pub const SORT_OR_NON_STR_KEYS: Opt = SORT_KEYS | NON_STR_KEYS;

pub const NOT_PASSTHROUGH: Opt =
    !(PASSTHROUGH_DATETIME | PASSTHROUGH_DATACLASS | PASSTHROUGH_SUBCLASS);

pub const MAX_OPT: i32 = (APPEND_NEWLINE
    | INDENT_2
    | NAIVE_UTC
    | NON_STR_KEYS
    | OMIT_MICROSECONDS
    | PASSTHROUGH_DATETIME
    | PASSTHROUGH_DATACLASS
    | PASSTHROUGH_SUBCLASS
    | SERIALIZE_DATACLASS
    | SERIALIZE_NUMPY
    | SERIALIZE_UUID
    | SORT_KEYS
    | STRICT_INTEGER
    | UTC_Z) as i32;
