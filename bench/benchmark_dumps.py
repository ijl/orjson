# SPDX-License-Identifier: (Apache-2.0 OR MIT)
from json import loads as json_loads

import pytest

import orjson

from .data import fixtures, libraries
from .simple_thread_pool import SimpleThreadPool
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
@pytest.mark.parametrize("release_gil", [False, True])
@pytest.mark.parametrize("fixture", ["canada.json", "citm_catalog.json"])
def test_multithread(benchmark, threads, release_gil, fixture):
    benchmark.group = f"{fixture} serialization"
    benchmark.extra_info["fixture"] = fixture
    benchmark.extra_info["threads"] = threads
    benchmark.extra_info["release_gil"] = release_gil

    data_str = read_fixture(f"{fixture}.xz")
    data_batch = [orjson.loads(data_str) for _ in range(32)]
    option = orjson.OPT_RELEASE_GIL if release_gil else None
    close_thread_pool = None

    if threads > 1:
        thread_pool = close_thread_pool = SimpleThreadPool(threads)

        def fn(batch):
            for d in batch:
                thread_pool.add_task(orjson.dumps, d, option=option)
            thread_pool.wait_completion()

    else:

        def fn(batch):
            for d in batch:
                orjson.dumps(d, option=option)

    benchmark(fn, data_batch)

    if close_thread_pool:
        close_thread_pool.close()
