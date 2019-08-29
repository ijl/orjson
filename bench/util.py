# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import os
import orjson
import lzma

dirname = os.path.join(os.path.dirname(__file__), "../data")

STR_CACHE = {}

OBJ_CACHE = {}


def read_fixture_str(filename):
    if not filename in STR_CACHE:
        if filename.endswith(".xz"):
            with lzma.open(os.path.join(dirname, filename), "r") as fileh:
                STR_CACHE[filename] = fileh.read().decode("utf-8")
        else:
            with open(os.path.join(dirname, filename), "r") as fileh:
                STR_CACHE[filename] = fileh.read().decode("utf-8")
    return STR_CACHE[filename]


def read_fixture_obj(filename):
    if not filename in OBJ_CACHE:
        OBJ_CACHE[filename] = orjson.loads(read_fixture_str(filename))
    return OBJ_CACHE[filename]
