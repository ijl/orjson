# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson

try:
    from typing import TypedDict
except ImportError:
    from typing_extensions import TypedDict


class TypedDictTests(unittest.TestCase):
    def test_typeddict(self):
        """
        dumps() TypedDict
        """

        class TypedDict1(TypedDict):
            a: str
            b: int

        obj = TypedDict1(a="a", b=1)
        self.assertEqual(orjson.dumps(obj), b'{"a":"a","b":1}')
