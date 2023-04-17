# SPDX-License-Identifier: (Apache-2.0 OR MIT)
from json import loads as json_loads
from queue import Queue
from threading import Thread

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


class SimpleThreadPool:
    class Worker(Thread):
        def __init__(self, tasks: Queue):
            Thread.__init__(self, daemon=True)
            self.tasks = tasks
            self.start()

        def run(self):
            while True:
                task = self.tasks.get()
                try:
                    if task is None:
                        break
                    func, args, kwargs = task
                    func(*args, **kwargs)
                finally:
                    self.tasks.task_done()

    def __init__(self, num_threads: int):
        self.tasks: Queue = Queue()
        self.workers = [SimpleThreadPool.Worker(self.tasks) for _ in range(num_threads)]

    def add_task(self, func, *args, **kwargs):
        self.tasks.put((func, args, kwargs))

    def wait_completion(self):
        self.tasks.join()

    def close(self):
        self.tasks.join()
        for _ in self.workers:
            self.tasks.put(None)
        for worker in self.workers:
            worker.join()


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
