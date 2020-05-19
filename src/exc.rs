// SPDX-License-Identifier: (Apache-2.0 OR MIT)

pub const INVALID_STR: &str = "str is not valid UTF-8: surrogates not allowed";
pub const RECURSION_LIMIT_REACHED: &str = "Recursion limit reached";
pub const DATETIME_LIBRARY_UNSUPPORTED: &str = "datetime's timezone library is not supported: use datetime.timezone.utc, pendulum, pytz, or dateutil";
pub const TIME_HAS_TZINFO: &str = "datetime.time must not have tzinfo set";
pub const KEY_MUST_BE_STR: &str = "Dict key must be str";
