# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import os
import lzma
from pathlib import Path

import orjson

dirname = os.path.join(os.path.dirname(__file__), "../data")

STR_CACHE = {}

OBJ_CACHE = {}


os.sched_setaffinity(os.getpid(), {0, 1})


def read_fixture_str(filename):
    if not filename in STR_CACHE:
        path = Path(dirname, filename)
        if path.suffix == ".xz":
            contents = lzma.decompress(path.read_bytes())
        else:
            contents = path.read_bytes()
        STR_CACHE[filename] = contents.decode("utf-8")
    return STR_CACHE[filename]


def read_fixture_obj(filename):
    if not filename in OBJ_CACHE:
        OBJ_CACHE[filename] = orjson.loads(read_fixture_str(filename))
    return OBJ_CACHE[filename]
