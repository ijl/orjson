# SPDX-License-Identifier: (Apache-2.0 OR MIT)

from json import loads as json_loads

import pytest

from .data import fixtures, libraries
from .util import read_fixture_str


@pytest.mark.parametrize("library", libraries)
@pytest.mark.parametrize("fixture", fixtures)
def test_dumps(benchmark, fixture, library):
    dumper, loader = libraries[library]
    benchmark.group = f"{fixture} serialization"
    benchmark.extra_info["lib"] = library
    data = read_fixture_str(f"{fixture}.xz")
    benchmark.extra_info["correct"] = json_loads(dumper(data)) == data
    benchmark(dumper, data)
