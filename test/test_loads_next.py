# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import pytest
import orjson


def loads_multiple(data):
    if isinstance(data, str):
        data = data.encode('utf-8')
    offset = 0
    while offset < len(data):
        result, bytes_read = orjson.loads_next(data[offset:])
        yield result
        offset += bytes_read


@pytest.mark.parametrize(
    ("data", "expected", "bytes_read"),
    [
        (b'{"a": 1}/* trailing trash */', {"a": 1}, 8),
        (b'{"a": 1}   ', {"a": 1}, 11),  # Consumes trailing spaces
        (b' {"a": 1}   ', {"a": 1}, 12),  # Consumes leading + trailing spaces
        (b'{"a": 1}{"b": 24}', {"a": 1}, 8),  # Stops before next object
        (bytearray(b'{"a": 42}'), {"a": 42}, 9),  # bytearray input
        (memoryview(b'  {"x": 67}  '), {"x": 67}, 13),  # memoryview input
    ],
)
def test_loads_next(data, expected, bytes_read):
    assert orjson.loads_next(data) == (expected, bytes_read)


@pytest.mark.parametrize(("data", "expected"), [
    (b'{"a": 1}{"b": 24}', [{"a": 1}, {"b": 24}]),  # Concatenated complex values
    (b'{"a": 1}  \n\t  {"b": 2}', [{"a": 1}, {"b": 2}]),  # With whitespace
    (b'[1,2,3]{"key":"value"}true"string"null', [[1, 2, 3], {"key": "value"}, True, "string", None]),  # Mixed types
    (b'{"a":1}\n{"b":2}\n{"c":3}\n', [{"a": 1}, {"b": 2}, {"c": 3}]),  # NDJSON
    (b'{"a":{"b":{"c":{"d":1}}}}', [{"a": {"b": {"c": {"d": 1}}}}]),  # Nested structures
    (b'[{"a":1},{"b":2},{"c":3}][][]', [[{"a": 1}, {"b": 2}, {"c": 3}], [], []]),  # Arrays with objects
    (b'{\n"a": 1,\n"b": 2,\n"c": 3\n}[1,\n2,\n3]', [{"a": 1, "b": 2, "c": 3}, [1, 2, 3]]),  # Pretty formatted
    (b'"first""second""third"', ["first", "second", "third"]),  # Consecutive strings
    (b'123 -456 789', [123, -456, 789]),  # Consecutive numbers
])
def test_loads_multiple(data, expected):
    assert list(loads_multiple(data)) == expected


@pytest.mark.parametrize(("data", "error_type"), [
    ('{"a": 1}', TypeError),  # str input
    (b'{invalid}', orjson.JSONDecodeError),  # Invalid JSON
    (b'{"a": 1', orjson.JSONDecodeError),  # Incomplete JSON
    (b'', orjson.JSONDecodeError),  # Empty input
    (b'   \n\t  ', orjson.JSONDecodeError),  # Only whitespace
    ('"törkylempijävongahdus"'.encode('iso-8859-1'), orjson.JSONDecodeError),  # Invalid UTF-8
])
def test_loads_next_error(data, error_type):
    with pytest.raises(error_type):
        orjson.loads_next(data)
