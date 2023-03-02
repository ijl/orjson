# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import pytest

import orjson


class TestCircular:
    def test_circular_dict(self):
        """
        dumps() circular reference dict
        """
        obj = {}  # type: ignore
        obj["obj"] = obj
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(obj)

    def test_circular_list(self):
        """
        dumps() circular reference list
        """
        obj = []  # type: ignore
        obj.append(obj)  # type: ignore
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(obj)

    def test_circular_nested(self):
        """
        dumps() circular reference nested dict, list
        """
        obj = {}  # type: ignore
        obj["list"] = [{"obj": obj}]
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(obj)
