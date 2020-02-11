# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime
import inspect
import json
import unittest

import orjson

SIMPLE_TYPES = (1, 1.0, -1, None, "str", True, False)


class ApiTests(unittest.TestCase):
    def test_loads_trailing(self):
        """
        loads() handles trailing whitespace
        """
        self.assertEqual(orjson.loads("{}\n\t "), {})

    def test_loads_trailing_invalid(self):
        """
        loads() handles trailing invalid
        """
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, "{}\n\t a")

    def test_simple_json(self):
        """
        dumps() equivalent to json on simple types
        """
        for obj in SIMPLE_TYPES:
            self.assertEqual(orjson.dumps(obj), json.dumps(obj).encode("utf-8"))

    def test_simple_round_trip(self):
        """
        dumps(), loads() round trip on simple types
        """
        for obj in SIMPLE_TYPES:
            self.assertEqual(orjson.loads(orjson.dumps(obj)), obj)

    def test_loads_type(self):
        """
        loads() invalid type
        """
        for val in (1, 3.14, [], {}, None):
            self.assertRaises(orjson.JSONDecodeError, orjson.loads, val)

    def test_loads_recursion(self):
        """
        loads() recursion limit
        """
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, "[" * (1024 * 1024))

    def test_version(self):
        """
        __version__
        """
        self.assertRegex(orjson.__version__, r"^\d+\.\d+(\.\d+)?$")

    def test_valueerror(self):
        """
        orjson.JSONDecodeError is a subclass of ValueError
        """
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, "{")
        self.assertRaises(ValueError, orjson.loads, "{")

    def test_option_not_int(self):
        """
        dumps() option not int or None
        """
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(True, option=True)

    def test_option_invalid_int(self):
        """
        dumps() option invalid 64-bit number
        """
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(True, option=9223372036854775809)

    def test_option_range_low(self):
        """
        dumps() option out of range low
        """
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(True, option=0)

    def test_option_range_high(self):
        """
        dumps() option out of range high
        """
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(True, option=1 << 7)

    def test_opts_multiple(self):
        """
        dumps() multiple option
        """
        self.assertEqual(
            orjson.dumps(
                [1, datetime.datetime(2000, 1, 1, 2, 3, 4)],
                option=orjson.OPT_STRICT_INTEGER | orjson.OPT_NAIVE_UTC,
            ),
            b'[1,"2000-01-01T02:03:04+00:00"]',
        )

    def test_dumps_signature(self):
        """
        dumps() valid __text_signature__
        """
        self.assertEqual(
            str(inspect.signature(orjson.dumps)), "(obj, /, default, option)"
        )

    def test_loads_signature(self):
        """
        loads() valid __text_signature__
        """
        self.assertEqual(str(inspect.signature(orjson.loads)), "(obj, /)")
