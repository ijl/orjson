// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use pyo3::create_exception;
use pyo3::import_exception;

pub const INVALID_STR: &str = "str is not valid UTF-8: surrogates not allowed";
pub const RECURSION_LIMIT_REACHED: &str = "Recursion limit reached";

import_exception!(json, JSONDecodeError);

create_exception!(orjson, JSONEncodeError, pyo3::exceptions::TypeError);
