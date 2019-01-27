# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime
import unittest
import json

import orjson


SIMPLE_TYPES = (1, 1.0, -1, None, "str", True, False)


class ApiTests(unittest.TestCase):

    def test_loads_trailing(self):
        """
        loads() handles trailing whitespace
        """
        self.assertEqual(orjson.loads('{}\n\t '), {})

    def test_loads_trailing_invalid(self):
        """
        loads() handles trailing invalid
        """
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, '{}\n\t a')

    def test_simple_json(self):
        """
        dumps() equivalent to json on simple types
        """
        for obj in SIMPLE_TYPES:
            self.assertEqual(orjson.dumps(obj), json.dumps(obj).encode('utf-8'))

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
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, '[' * (1024 * 1024))

    def test_version(self):
        """
        __version__
        """
        self.assertRegex(orjson.__version__, r'^\d+\.\d+(\.\d+)?$')

    def test_valueerror(self):
        """
        orjson.JSONDecodeError is a subclass of ValueError
        """
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, '{')
        self.assertRaises(ValueError, orjson.loads, '{')

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
            orjson.dumps(True, option=4)


    def test_opts_multiple(self):
        """
        dumps() multiple option
        """
        self.assertEqual(
            orjson.dumps(
                [1, datetime.datetime.fromtimestamp(4123518902)],
                option=orjson.OPT_STRICT_INTEGER | orjson.OPT_NAIVE_UTC,
            ),
            b'[1,"2100-09-01T21:55:02+00:00"]'
        )
