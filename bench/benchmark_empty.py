# SPDX-License-Identifier: (Apache-2.0 OR MIT)

from json import loads as json_loads

import pytest

from .data import libraries


@pytest.mark.parametrize("data", ["[]", "{}", '""'])
@pytest.mark.parametrize("library", libraries)
def test_empty(benchmark, data, library):
    dumper, loader = libraries[library]
    benchmark.extra_info["correct"] = json_loads(dumper(loader(data))) == json_loads(
        data
    )
    benchmark(loader, data)
