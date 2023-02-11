# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import orjson


class TestDict:
    def test_dict_pop_replace_first(self):
        """Test pop and replace a first key in a dict with other keys."""
        data = {"id": "any", "other": "any"}
        data.pop("id")
        assert orjson.dumps(data) == b'{"other":"any"}'
        data["id"] = "new"
        assert orjson.dumps(data) == b'{"other":"any","id":"new"}'

    def test_dict_pop_replace_last(self):
        """Test pop and replace a last key in a dict with other keys."""
        data = {"other": "any", "id": "any"}
        data.pop("id")
        assert orjson.dumps(data) == b'{"other":"any"}'
        data["id"] = "new"
        assert orjson.dumps(data) == b'{"other":"any","id":"new"}'

    def test_dict_pop(self):
        """Test pop and replace a key in a dict with no other keys."""
        data = {"id": "any"}
        data.pop("id")
        assert orjson.dumps(data) == b"{}"
        data["id"] = "new"
        assert orjson.dumps(data) == b'{"id":"new"}'

    def test_in_place(self):
        """Mutate dict in-place"""
        data = {"id": "any", "static": "msg"}
        data["id"] = "new"
        assert orjson.dumps(data) == b'{"id":"new","static":"msg"}'

    def test_dict_0xff(self):
        """dk_size <= 0xff"""
        data = {str(idx): idx for idx in range(0, 0xFF)}
        data.pop("112")
        data["112"] = 1
        data["113"] = 2
        assert orjson.loads(orjson.dumps(data)) == data

    def test_dict_0xffff(self):
        """dk_size <= 0xffff"""
        data = {str(idx): idx for idx in range(0, 0xFFFF)}
        data.pop("112")
        data["112"] = 1
        data["113"] = 2
        assert orjson.loads(orjson.dumps(data)) == data

    def test_dict_dict(self):
        class C:
            def __init__(self):
                self.a = 0
                self.b = 1

        assert orjson.dumps(C().__dict__) == b'{"a":0,"b":1}'
