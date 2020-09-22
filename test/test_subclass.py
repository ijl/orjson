# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import collections
import json
import unittest

import orjson


class SubStr(str):
    pass


class SubInt(int):
    pass


class SubDict(dict):
    pass


class SubList(list):
    pass


class SubFloat(float):
    pass


class SubTuple(tuple):
    pass


class SubclassTests(unittest.TestCase):
    def test_subclass_str(self):
        self.assertEqual(
            orjson.dumps(SubStr("zxc")),
            b'"zxc"',
        )

    def test_subclass_str_invalid(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(SubStr("\ud800"))

    def test_subclass_int(self):
        self.assertEqual(orjson.dumps(SubInt(1)), b"1")

    def test_subclass_int_64(self):
        for val in (9223372036854775807, -9223372036854775807):
            self.assertEqual(orjson.dumps(SubInt(val)), str(val).encode("utf-8"))

    def test_subclass_int_53(self):
        for val in (9007199254740992, -9007199254740992):
            with self.assertRaises(orjson.JSONEncodeError):
                orjson.dumps(SubInt(val), option=orjson.OPT_STRICT_INTEGER)

    def test_subclass_dict(self):
        self.assertEqual(
            orjson.dumps(SubDict({"a": "b"})),
            b'{"a":"b"}',
        )

    def test_subclass_list(self):
        self.assertEqual(
            orjson.dumps(SubList(["a", "b"])),
            b'["a","b"]',
        )
        ref = [True] * 512
        self.assertEqual(orjson.loads(orjson.dumps(SubList(ref))), ref)

    def test_subclass_float(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(SubFloat(1.1))
        self.assertEqual(
            json.dumps(SubFloat(1.1)),
            "1.1",
        )

    def test_subclass_tuple(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(SubTuple((1, 2)))
        self.assertEqual(
            json.dumps(SubTuple((1, 2))),
            "[1, 2]",
        )

    def test_namedtuple(self):
        Point = collections.namedtuple("Point", ["x", "y"])
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(Point(1, 2))

    def test_subclass_circular_dict(self):
        obj = SubDict({})
        obj["obj"] = obj
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj)

    def test_subclass_circular_list(self):
        obj = SubList([])
        obj.append(obj)
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj)

    def test_subclass_circular_nested(self):
        obj = SubDict({})
        obj["list"] = SubList([{"obj": obj}])
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj)


class SubclassPassthroughTests(unittest.TestCase):
    def test_subclass_str(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(SubStr("zxc"), option=orjson.OPT_PASSTHROUGH_SUBCLASS)

    def test_subclass_int(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(SubInt(1), option=orjson.OPT_PASSTHROUGH_SUBCLASS)

    def test_subclass_dict(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(SubDict({"a": "b"}), option=orjson.OPT_PASSTHROUGH_SUBCLASS)

    def test_subclass_list(self):
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(SubList(["a", "b"]), option=orjson.OPT_PASSTHROUGH_SUBCLASS)
