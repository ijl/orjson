# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import dataclasses
import datetime
import gc
import random
import unittest
from typing import List

import pytz

try:
    import psutil
except ImportError:
    psutil = None  # type: ignore
import pytest

import orjson

try:
    import numpy
except ImportError:
    numpy = None  # type: ignore

FIXTURE = '{"a":[81891289, 8919812.190129012], "b": false, "c": null, "d": "東京"}'


def default(obj):
    return str(obj)


@dataclasses.dataclass
class Member:
    id: int
    active: bool


@dataclasses.dataclass
class Object:
    id: int
    updated_at: datetime.datetime
    name: str
    members: List[Member]


DATACLASS_FIXTURE = [
    Object(
        i,
        datetime.datetime.now(datetime.timezone.utc)
        + datetime.timedelta(seconds=random.randint(0, 10000)),
        str(i) * 3,
        [Member(j, True) for j in range(0, 10)],
    )
    for i in range(100000, 101000)
]

MAX_INCREASE = 1048576  # 1MiB


class Unsupported:
    pass


class MemoryTests(unittest.TestCase):
    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_loads(self):
        """
        loads() memory leak
        """
        proc = psutil.Process()
        gc.collect()
        val = orjson.loads(FIXTURE)
        mem = proc.memory_info().rss
        for _ in range(10000):
            val = orjson.loads(FIXTURE)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_loads_memoryview(self):
        """
        loads() memory leak using memoryview
        """
        proc = psutil.Process()
        gc.collect()
        fixture = FIXTURE.encode("utf-8")
        val = orjson.loads(fixture)
        mem = proc.memory_info().rss
        for _ in range(10000):
            val = orjson.loads(memoryview(fixture))
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_dumps(self):
        """
        dumps() memory leak
        """
        proc = psutil.Process()
        gc.collect()
        fixture = orjson.loads(FIXTURE)
        val = orjson.dumps(fixture)
        mem = proc.memory_info().rss
        for _ in range(10000):
            val = orjson.dumps(fixture)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_loads_exc(self):
        """
        loads() memory leak exception without a GC pause
        """
        proc = psutil.Process()
        gc.disable()
        mem = proc.memory_info().rss
        n = 10000
        i = 0
        for _ in range(n):
            try:
                orjson.loads("")
            except orjson.JSONDecodeError:
                i += 1
        assert n == i
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)
        gc.enable()

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_dumps_exc(self):
        """
        dumps() memory leak exception without a GC pause
        """
        proc = psutil.Process()
        gc.disable()
        data = Unsupported()
        mem = proc.memory_info().rss
        n = 10000
        i = 0
        for _ in range(n):
            try:
                orjson.dumps(data)
            except orjson.JSONEncodeError:
                i += 1
        assert n == i
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)
        gc.enable()

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_dumps_default(self):
        """
        dumps() default memory leak
        """
        proc = psutil.Process()
        gc.collect()
        fixture = orjson.loads(FIXTURE)

        class Custom:
            def __init__(self, name):
                self.name = name

            def __str__(self):
                return f"{self.__class__.__name__}({self.name})"

        fixture["custom"] = Custom("orjson")
        val = orjson.dumps(fixture, default=default)
        mem = proc.memory_info().rss
        for _ in range(10000):
            val = orjson.dumps(fixture, default=default)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_dumps_dataclass(self):
        """
        dumps() dataclass memory leak
        """
        proc = psutil.Process()
        gc.collect()
        val = orjson.dumps(DATACLASS_FIXTURE)
        mem = proc.memory_info().rss
        for _ in range(100):
            val = orjson.dumps(DATACLASS_FIXTURE)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_dumps_pytz_tzinfo(self):
        """
        dumps() pytz tzinfo memory leak
        """
        proc = psutil.Process()
        gc.collect()
        dt = datetime.datetime.now()
        val = orjson.dumps(pytz.UTC.localize(dt))
        mem = proc.memory_info().rss
        for _ in range(50000):
            val = orjson.dumps(pytz.UTC.localize(dt))
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    def test_memory_loads_keys(self):
        """
        loads() memory leak with number of keys causing cache eviction
        """
        proc = psutil.Process()
        gc.collect()
        fixture = {"key_%s" % idx: "value" for idx in range(1024)}
        self.assertEqual(len(fixture), 1024)
        val = orjson.dumps(fixture)
        loaded = orjson.loads(val)
        mem = proc.memory_info().rss
        for _ in range(100):
            loaded = orjson.loads(val)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    @pytest.mark.skipif(
        psutil is None, reason="psutil install broken on win, python3.9, Azure"
    )
    @pytest.mark.skipif(numpy is None, reason="numpy is not installed")
    def test_memory_dumps_numpy(self):
        """
        dumps() dataclass memory leak
        """
        proc = psutil.Process()
        gc.collect()
        fixture = numpy.random.rand(4, 4, 4)
        val = orjson.dumps(fixture, option=orjson.OPT_SERIALIZE_NUMPY)
        mem = proc.memory_info().rss
        for _ in range(100):
            val = orjson.dumps(fixture, option=orjson.OPT_SERIALIZE_NUMPY)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)
