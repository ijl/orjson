# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import lzma
import os
from pathlib import Path

dirname = os.path.join(os.path.dirname(__file__), "../data")


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
    return read_fixture_bytes(filename, subdir).decode("utf-8")
