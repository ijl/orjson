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
        path = Path(dirname, filename)
    else:
        path = Path(dirname, subdir, filename)
    if path.suffix == ".xz":
        contents = lzma.decompress(path.read_bytes())
    else:
        contents = path.read_bytes()
    return contents


def read_fixture_str(filename, subdir=None):
    if filename not in STR_CACHE:
        STR_CACHE[filename] = read_fixture_bytes(filename, subdir).decode("utf-8")
    return STR_CACHE[filename]


def read_fixture_obj(filename):
    if filename not in OBJ_CACHE:
        OBJ_CACHE[filename] = orjson.loads(read_fixture_str(filename))
    return OBJ_CACHE[filename]
