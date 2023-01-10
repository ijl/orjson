# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import orjson

from .util import read_fixture_bytes

def test_issue331_1():
    as_bytes = read_fixture_bytes("issue331_1.json.xz")
    as_obj = orjson.loads(as_bytes)
    for i in range(1000):
        assert orjson.loads(orjson.dumps(as_obj)) == as_obj

def test_issue331_2():
    as_bytes = read_fixture_bytes("issue331_2.json.xz")
    as_obj = orjson.loads(as_bytes)
    for i in range(1000):
        assert orjson.loads(orjson.dumps(as_obj)) == as_obj
