# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest
from textwrap import dedent

import orjson


class DictPrettyFormatTests(unittest.TestCase):
    def _strip_dedent(self, text):
        return dedent(text).strip()

    def _assert_equal_str(self, a, b):
        if isinstance(a, bytes):
            a = a.decode('utf8')
        if isinstance(b, bytes):
            b = b.decode('utf8')
        self.assertEqual(a, b)

    def test_pretty_format(self):
        data = {
            "list": [
                "bloop",
                1,
                2,
                "3",
            ],
        }
        out = orjson.dumps(data, option=orjson.OPT_PRETTY)
        expected = self._strip_dedent(
            """
            {
              "list": [
                "bloop",
                1,
                2,
                "3"
              ]
            }
            """
        )
        self._assert_equal_str(out, expected)

    def test_pretty_sorted_format(self):
        data = {
            "x": "b",
            "list": [
                1,
                2,
            ],
            "a_longer": [
                1,
                2,
            ],
            "d": 5,
            "a": 9,
        }
        out = orjson.dumps(data, option=orjson.OPT_PRETTY | orjson.OPT_SORT_KEYS)
        expected = self._strip_dedent(
            """
            {
              "a": 9,
              "a_longer": [
                1,
                2
              ],
              "d": 5,
              "list": [
                1,
                2
              ],
              "x": "b"
            }
            """
        )
        self._assert_equal_str(out, expected)
