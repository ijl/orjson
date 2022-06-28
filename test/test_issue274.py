# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import orjson


def test_pop():
    data = {"id": "any", "static": "msg"}
    data.pop("id")
    data["id"] = "new"
    # not b'{"static":"msg","static":"msg"}'
    assert orjson.dumps(data) == b'{"static":"msg","id":"new"}'


def test_in_place():
    # not an issue
    data = {"id": "any", "static": "msg"}
    data["id"] = "new"
    assert orjson.dumps(data) == b'{"id":"new","static":"msg"}'
