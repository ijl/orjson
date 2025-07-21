# SPDX-License-Identifier: (Apache-2.0 OR MIT)


import orjson


class TestLoadNanAsNoneTests:
    def test_nan_inf_in_object(self):
        """
        Test that NaN, Infinity, and -Infinity are loaded as None in dict values with OPT_NAN_AS_NULL.
        """
        data = b'{"a": nan, "b": Infinity, "c": -Infinity, "d": NaN, "e": 2}'
        result = orjson.loads(data, option=orjson.OPT_NAN_AS_NULL)
        assert result == {"a": None, "b": None, "c": None, "d": None, "e": 2}

    def test_nan_inf_nested_list(self):
        """
        Test that NaN, Infinity, and -Infinity are loaded as None in nested lists with OPT_NAN_AS_NULL.
        """
        data = b"[1, [nan, 2, [Infinity, -Infinity, NaN]], 3]"
        result = orjson.loads(data, option=orjson.OPT_NAN_AS_NULL)
        assert result == [1, [None, 2, [None, None, None]], 3]

    def test_nan_inf_nested_object(self):
        """
        Test that NaN, Infinity, and -Infinity are loaded as None in nested dicts with OPT_NAN_AS_NULL.
        """
        data = b'{"x": {"y": nan, "z": [Infinity, -Infinity, NaN]}, "w": 5}'
        result = orjson.loads(data, option=orjson.OPT_NAN_AS_NULL)
        assert result == {"x": {"y": None, "z": [None, None, None]}, "w": 5}

    def test_nan_inf_mixed(self):
        """
        Test that NaN, Infinity, and -Infinity are loaded as None in mixed structures with OPT_NAN_AS_NULL.
        """
        data = b'[{"a": nan}, [Infinity, {"b": -Infinity}], NaN]'
        result = orjson.loads(data, option=orjson.OPT_NAN_AS_NULL)
        assert result == [{"a": None}, [None, {"b": None}], None]
