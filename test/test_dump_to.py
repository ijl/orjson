# SPDX-License-Identifier: (Apache-2.0 OR MIT)
import dataclasses
import io
import os
import tempfile
import pytest
import orjson


@dataclasses.dataclass(frozen=True)
class AssertRoundtripResult:
    chunks: list[bytes] = dataclasses.field(default_factory=list)

    @property
    def result(self) -> bytes:
        return b"".join(self.chunks)


def assert_roundtrip(data, *, assert_n_chunks=None, **kwargs) -> AssertRoundtripResult:
    """Helper to assert that data roundtrips correctly"""
    res = AssertRoundtripResult()
    orjson.dump_to(data, res.chunks.append, **kwargs)
    assert orjson.loads(res.result) == data, "Data did not roundtrip correctly"
    dumps_kwargs = kwargs.copy()
    dumps_kwargs.pop("buffer_size", None)
    assert orjson.dumps(data, **dumps_kwargs) == res.result, "dump_to and dumps outputs differ"
    if assert_n_chunks is not None:
        assert len(res.chunks) == assert_n_chunks
    return res


@pytest.mark.parametrize("data", [
    {"name": "Alice", "age": 30},
    [],
    {},
    {
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
        ],
        "meta": {"count": 2}
    },
    {"emoji": "🚀", "text": "Hello 世界"},
    {"special": "quotes\"and\\backslashes\nand\nnewlines\t\ttabs"}
])
def test_basic(data):
    assert_roundtrip(data, buffer_size=1048576, assert_n_chunks=1)


def test_multiple_chunks():
    """Large array should trigger multiple flushes"""
    data = [{"id": i, "value": "x" * 1000} for i in range(2000)]
    res = assert_roundtrip(data, buffer_size=65536)
    assert len(res.chunks) > 1


def test_single_large_string():
    """Test that a single large string exceeding buffer works"""
    res = assert_roundtrip({"large": ("\n" * (1025 * 1024))}, buffer_size=1024 * 1024)
    assert any(len(c) > 1024 * 1024 for c in res.chunks)


def test_various_options():
    """Test with various options"""
    res = assert_roundtrip(
        {"z": 1, "a": 2, "b": 3},
        option=orjson.OPT_INDENT_2 | orjson.OPT_SORT_KEYS | orjson.OPT_APPEND_NEWLINE,
    )
    result = res.result
    assert b"  " in result  # Should have indentation
    assert result.endswith(b"\n")  # Should end with newline
    assert result.index(b"\"a\"") < result.index(b"\"b\"") < result.index(b"\"z\"")  # Sorted keys


def test_with_default():
    """Test custom default handler"""
    res = AssertRoundtripResult()
    orjson.dump_to({"obj": b"w\x00gh"}, res.chunks.append, default=str)
    assert res.result == b'{"obj":"b\'w\\\\x00gh\'"}'


def test_callback_raises_exception():
    """Test that callback exceptions are propagated"""

    def failing_callback(_chunk):
        raise ValueError("Ow, that hurts!")

    with pytest.raises(orjson.JSONEncodeError) as ve:
        orjson.dump_to({"x": 1}, failing_callback)
    assert str(ve.value) == "Callback function raised an exception"
    assert str(ve.value.__cause__) == "Ow, that hurts!"


def test_non_callable_callback():
    """Test that non-callable callback raises TypeError"""
    with pytest.raises(TypeError, match="callback must be callable"):
        orjson.dump_to({"x": 1}, "not a function")


def test_missing_arguments():
    """Test missing required arguments"""
    with pytest.raises(TypeError, match="missing required positional"):
        orjson.dump_to({"x": 1})


@pytest.mark.parametrize("size", [-10, 0])
def test_invalid_buffer_size(size):
    """Test invalid buffer size"""
    with pytest.raises(TypeError, match="buffer_size must be"):
        assert_roundtrip({"x": 1}, buffer_size=-1)


def test_unserializable_object():
    """Test that unserializable objects raise proper errors"""
    with pytest.raises(TypeError):
        assert_roundtrip({"obj": object()})


def test_write_to_file(tmp_path):
    """Test writing directly to a file"""
    tmp_file = tmp_path / "test.json"
    with tmp_file.open("wb") as f:
        data = [{"id": i, "value": f"item_{i}"} for i in range(100)]
        orjson.dump_to(data, f.write, buffer_size=2048)
    assert orjson.loads(tmp_file.read_bytes()) == data


def test_write_to_bytesio():
    """Test writing to BytesIO"""
    buffer = io.BytesIO()
    data = {"x": 1, "y": 2, "z": 3}
    orjson.dump_to(data, buffer.write)
    result = orjson.loads(buffer.getbuffer())
    assert result == data
