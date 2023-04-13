# SPDX-License-Identifier: (Apache-2.0 OR MIT)
from concurrent.futures import ThreadPoolExecutor, wait
from json import loads as json_loads

import pytest

import orjson

from .data import fixtures, libraries
from .util import read_fixture, read_fixture_obj


@pytest.mark.parametrize("library", libraries)
@pytest.mark.parametrize("fixture", fixtures)
def test_dumps(benchmark, fixture, library):
    dumper, loader = libraries[library]
    benchmark.group = f"{fixture} serialization"
    benchmark.extra_info["lib"] = library
    data = read_fixture_obj(f"{fixture}.xz")
    benchmark.extra_info["correct"] = json_loads(dumper(data)) == data
    benchmark(dumper, data)


@pytest.mark.parametrize("threads", [1, 2, 3, 4])
@pytest.mark.parametrize("fixture", fixtures)
def test_multithread(benchmark, threads, fixture):
    benchmark.group = f"{fixture} serialization"
    benchmark.extra_info["threads"] = threads
    benchmark.extra_info["fixture"] = fixture
    data_str = read_fixture(f"{fixture}.xz")
    benchmark.extra_info["correct"] = orjson.loads(data_str) == json_loads(data_str)

    with ThreadPoolExecutor(max_workers=threads) as pool:
        if threads > 1:

            def fn(batch):
                futures = [
                    pool.submit(lambda: orjson.dumps(d, option=orjson.OPT_NO_GIL))
                    for d in batch
                ]
                wait(futures)

        else:

            def fn(batch):
                for d in batch:
                    orjson.dumps(d)

        data_batch = [orjson.loads(data_str) for _ in range(30)]
        benchmark(fn, data_batch)
