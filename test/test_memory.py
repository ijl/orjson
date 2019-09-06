# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import gc
import unittest
import uuid

import pytest
import psutil
import orjson


FIXTURE = '{"a":[81891289, 8919812.190129012], "b": false, "c": null, "d": "東京"}'


def default(obj):
    return str(obj)


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
        self.assertTrue(proc.memory_info().rss <= mem + 1024)

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
        self.assertTrue(proc.memory_info().rss <= mem + 1024)

    def test_memory_dumps_default(self):
        """
        dumps() default memory leak
        """
        proc = psutil.Process()
        gc.collect()
        fixture = orjson.loads(FIXTURE)
        fixture["custom"] = uuid.uuid4()
        val = orjson.dumps(fixture, default=default)
        mem = proc.memory_info().rss
        for _ in range(10000):
            val = orjson.dumps(fixture, default=default)
        gc.collect()
        self.assertTrue(proc.memory_info().rss <= mem + 1024)
