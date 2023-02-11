# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import lzma
import os
from pathlib import Path
from typing import Any, Dict

import orjson

dirname = os.path.join(os.path.dirname(__file__), "../data")

STR_CACHE: Dict[str, str] = {}

OBJ_CACHE: Dict[str, Any] = {}


def read_fixture_bytes(filename, subdir=None):
    if subdir is None:
        parts = (dirname, filename)
    else:
        parts = (dirname, subdir, filename)
    path = Path(*parts)
    if path.suffix == ".xz":
        contents = lzma.decompress(path.read_bytes())
    else:
        contents = path.read_bytes()
    return contents


def read_fixture_str(filename, subdir=None):
    if not filename in STR_CACHE:
        STR_CACHE[filename] = read_fixture_bytes(filename, subdir).decode("utf-8")
    return STR_CACHE[filename]


def read_fixture_obj(filename):
    if not filename in OBJ_CACHE:
        OBJ_CACHE[filename] = orjson.loads(read_fixture_str(filename))
    return OBJ_CACHE[filename]
