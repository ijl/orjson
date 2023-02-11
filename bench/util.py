# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import lzma
import os
from functools import lru_cache
from pathlib import Path
from typing import Any

import orjson

dirname = os.path.join(os.path.dirname(__file__), "../data")

if hasattr(os, "sched_setaffinity"):
    os.sched_setaffinity(os.getpid(), {0, 1})


@lru_cache(maxsize=None)
def read_fixture(filename: str) -> bytes:
    path = Path(dirname, filename)
    if path.suffix == ".xz":
        contents = lzma.decompress(path.read_bytes())
    else:
        contents = path.read_bytes()
    return contents


@lru_cache(maxsize=None)
def read_fixture_obj(filename: str) -> Any:
    return orjson.loads(read_fixture(filename))
