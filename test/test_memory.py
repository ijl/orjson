# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import dataclasses
import datetime
import gc
import random
import unittest
from typing import List

import orjson
import psutil

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


class MemoryTests(unittest.TestCase):
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
                return "%s(%s)" % (self.__class__.__name__, self.name)

        fixture["custom"] = Custom("orjson")
        val = orjson.dumps(fixture, default=default)
        mem = proc.memory_info().rss
        for _ in range(10000):
            val = orjson.dumps(fixture, default=default)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

    def test_memory_dumps_dataclass(self):
        """
        dumps() dataclass memory leak
        """
        proc = psutil.Process()
        gc.collect()
        val = orjson.dumps(DATACLASS_FIXTURE, option=orjson.OPT_SERIALIZE_DATACLASS)
        mem = proc.memory_info().rss
        for _ in range(100):
            val = orjson.dumps(DATACLASS_FIXTURE, option=orjson.OPT_SERIALIZE_DATACLASS)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + MAX_INCREASE)

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
