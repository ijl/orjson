# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson


class CircularTests(unittest.TestCase):
    def test_circular_dict(self):
        """
        dumps() circular reference dict
        """
        obj = {}
        obj["obj"] = obj
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj)

    def test_circular_list(self):
        """
        dumps() circular reference list
        """
        obj = []
        obj.append(obj)
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj)

    def test_circular_nested(self):
        """
        dumps() circular reference nested dict, list
        """
        obj = {}
        obj["list"] = [{"obj": obj}]
        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj)
