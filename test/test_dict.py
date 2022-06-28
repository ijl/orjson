# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson


class DictTests(unittest.TestCase):
    def test_dict_pop_replace_first(self):
        """Test pop and replace a first key in a dict with other keys."""
        data = {"id": "any", "other": "any"}
        data.pop("id")
        data["id"] = "new"
        self.assertEqual(orjson.dumps(data), b'{"other":"any","id":"new"}')

    def test_dict_pop_replace_last(self):
        """Test pop and replace a last key in a dict with other keys."""
        data = {"other": "any", "id": "any"}
        data.pop("id")
        data["id"] = "new"
        self.assertEqual(orjson.dumps(data), b'{"other":"any","id":"new"}')

    def test_dict_pop(self):
        """Test pop and replace a key in a dict with no other keys."""
        data = {"id": "any"}
        data.pop("id")
        data["id"] = "new"
        self.assertEqual(orjson.dumps(data), b'{"id":"new"}')
