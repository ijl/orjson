# SPDX-License-Identifier: (Apache-2.0 OR MIT)


import dataclasses
import uuid
import zoneinfo
from datetime import datetime, timezone

import pytest

import orjson

from .util import numpy


class TestBigIntegerTests:
    def test_big_integer_dumps(self):
        """
        Big integers are serialized as strings
        """

        with pytest.raises(TypeError):
            orjson.dumps(100000000000000000001)

        with pytest.raises(TypeError):
            orjson.dumps(-100000000000000000001)

        assert (
            orjson.dumps(100000000000000000001, option=orjson.OPT_BIG_INTEGER)
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(-100000000000000000001, option=orjson.OPT_BIG_INTEGER)
            == b"-100000000000000000001"
        )

    def test_big_integer_loads(self):
        """
        Big integers are deserialized as integers
        """

        assert orjson.loads(b"10000000000000000000") == 10000000000000000000
        assert orjson.loads(b"-10000000000000000000") == -10000000000000000000

        assert (
            orjson.loads(b"100000000000000000001", option=orjson.OPT_BIG_INTEGER)
            == 100000000000000000001
        )
        assert (
            orjson.loads(b"-100000000000000000001", option=orjson.OPT_BIG_INTEGER)
            == -100000000000000000001
        )

    def test_big_integers_dict_key_dumps(self):
        """
        Big integers as dict keys are serialized as strings
        """

        with pytest.raises(TypeError):
            orjson.dumps({100000000000000000001: True}, option=orjson.OPT_NON_STR_KEYS)

        with pytest.raises(TypeError):
            orjson.dumps({-100000000000000000001: True}, option=orjson.OPT_NON_STR_KEYS)

        assert (
            orjson.dumps(
                {100000000000000000001: True},
                option=orjson.OPT_NON_STR_KEYS | orjson.OPT_BIG_INTEGER,
            )
            == b'{"100000000000000000001":true}'
        )
        assert (
            orjson.dumps(
                {-100000000000000000001: True},
                option=orjson.OPT_NON_STR_KEYS | orjson.OPT_BIG_INTEGER,
            )
            == b'{"-100000000000000000001":true}'
        )

    def test_big_integers_dict_key_loads(self):
        """
        Big integers as dict keys are deserialized as integers
        """
        assert orjson.loads(b'{"10000000000000000000":true}') == {
            "10000000000000000000": True,
        }
        assert orjson.loads(b'{"-10000000000000000000":true}') == {
            "-10000000000000000000": True,
        }

        assert orjson.loads(b'{"100000000000000000001":true}') == {
            "100000000000000000001": True,
        }
        assert orjson.loads(b'{"-100000000000000000001":true}') == {
            "-100000000000000000001": True,
        }

        with pytest.raises(orjson.JSONDecodeError):
            orjson.loads(b"{10000000000000000000:true}")

    def test_big_integer_flag_combination_1(self):
        """
        OPT_BIG_INTEGER can be combined with other options 1
        """

        # OPT_INDENT_2
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_INDENT_2,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_INDENT_2,
            )
            == b"-100000000000000000001"
        )

        # OPT_INDENT_2 for dict
        assert (
            orjson.dumps(
                {"key": 100000000000000000001},
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_INDENT_2,
            )
            == b'{\n  "key": 100000000000000000001\n}'
        )
        assert (
            orjson.dumps(
                {"key": -100000000000000000001},
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_INDENT_2,
            )
            == b'{\n  "key": -100000000000000000001\n}'
        )

        # OPT_INDENT_2 for list
        assert (
            orjson.dumps(
                [100000000000000000001],
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_INDENT_2,
            )
            == b"[\n  100000000000000000001\n]"
        )
        assert (
            orjson.dumps(
                [-100000000000000000001],
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_INDENT_2,
            )
            == b"[\n  -100000000000000000001\n]"
        )

        # OPT_NON_STR_KEYS
        assert (
            orjson.dumps(
                1234567890123456789012345678901234567890,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_NON_STR_KEYS,
            )
            == b"1234567890123456789012345678901234567890"
        )
        assert (
            orjson.dumps(
                [1234567890123456789012345678901234567890],
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_NON_STR_KEYS,
            )
            == b"[1234567890123456789012345678901234567890]"
        )
        assert (
            orjson.dumps(
                {100000000000000000001: True},
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_NON_STR_KEYS,
            )
            == b'{"100000000000000000001":true}'
        )
        assert (
            orjson.dumps(
                {-100000000000000000001: True},
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_NON_STR_KEYS,
            )
            == b'{"-100000000000000000001":true}'
        )
        assert (
            orjson.dumps(
                {100000000000000000001: True},
                option=orjson.OPT_BIG_INTEGER
                | orjson.OPT_INDENT_2
                | orjson.OPT_NON_STR_KEYS,
            )
            == b'{\n  "100000000000000000001": true\n}'
        )
        assert (
            orjson.dumps(
                {-100000000000000000001: True},
                option=orjson.OPT_BIG_INTEGER
                | orjson.OPT_INDENT_2
                | orjson.OPT_NON_STR_KEYS,
            )
            == b'{\n  "-100000000000000000001": true\n}'
        )

        # OPT_SORT_KEYS for dict with non-str keys
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SORT_KEYS,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SORT_KEYS,
            )
            == b"-100000000000000000001"
        )
        assert (
            orjson.dumps(
                {100000000000000000001: True, -100000000000000000001: False},
                option=orjson.OPT_BIG_INTEGER
                | orjson.OPT_SORT_KEYS
                | orjson.OPT_NON_STR_KEYS,
            )
            == b'{"-100000000000000000001":false,"100000000000000000001":true}'
        )

        # OPT_APPEND_NEWLINE
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_APPEND_NEWLINE,
            )
            == b"100000000000000000001\n"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_APPEND_NEWLINE,
            )
            == b"-100000000000000000001\n"
        )

        # OPT_APPEND_NEWLINE for dict
        assert (
            orjson.dumps(
                {"key": 100000000000000000001},
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_APPEND_NEWLINE,
            )
            == b'{"key":100000000000000000001}\n'
        )
        assert (
            orjson.dumps(
                {"key": -100000000000000000001},
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_APPEND_NEWLINE,
            )
            == b'{"key":-100000000000000000001}\n'
        )

        # OPT_APPEND_NEWLINE for list
        assert (
            orjson.dumps(
                [100000000000000000001],
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_APPEND_NEWLINE,
            )
            == b"[100000000000000000001]\n"
        )
        assert (
            orjson.dumps(
                [-100000000000000000001],
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_APPEND_NEWLINE,
            )
            == b"[-100000000000000000001]\n"
        )

    def test_big_integer_flag_combination_2(self):
        """
        OPT_BIG_INTEGER can be combined with other options 2
        """

        # OPT_NAIVE_UTC
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_NAIVE_UTC,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_NAIVE_UTC,
            )
            == b"-100000000000000000001"
        )
        assert (
            orjson.dumps(
                datetime(2023, 1, 1, 12, 0, 0),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_NAIVE_UTC,
            )
            == b'"2023-01-01T12:00:00+00:00"'
        )

        # OPT_UTC_Z with datetime
        assert (
            orjson.dumps(
                datetime(2023, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_UTC_Z,
            )
            == b'"2023-01-01T12:00:00Z"'
        )

        # OPT_OMIT_MICROSECONDS
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_OMIT_MICROSECONDS,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_OMIT_MICROSECONDS,
            )
            == b"-100000000000000000001"
        )
        assert (
            orjson.dumps(
                datetime(2023, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_OMIT_MICROSECONDS,
            )
            == b'"2023-01-01T12:00:00+00:00"'
        )

        # OPT_PASSTHROUGH_DATETIME
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_DATETIME,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_DATETIME,
            )
            == b"-100000000000000000001"
        )

        with pytest.raises(TypeError):
            orjson.dumps(
                datetime(2023, 1, 1, 12, 0, 0, tzinfo=timezone.utc),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_DATETIME,
            )

        def default(obj):
            if isinstance(obj, datetime):
                return obj.strftime("%a, %d %b %Y %H:%M:%S GMT")
            raise TypeError

        assert (
            orjson.dumps(
                datetime(1970, 1, 1),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_DATETIME,
                default=default,
            )
            == b'"Thu, 01 Jan 1970 00:00:00 GMT"'
        )

    def test_big_integer_flag_combination_3(self):
        """
        OPT_BIG_INTEGER can be combined with other options 3
        """

        # OPT_UTC_Z
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_UTC_Z,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_UTC_Z,
            )
            == b"-100000000000000000001"
        )
        assert (
            orjson.dumps(
                datetime(1970, 1, 1, 0, 0, 0, tzinfo=zoneinfo.ZoneInfo("UTC")),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_UTC_Z,
            )
            == b'"1970-01-01T00:00:00Z"'
        )

        # OPT_PASSTHROUGH_SUBCLASS
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_SUBCLASS,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_SUBCLASS,
            )
            == b"-100000000000000000001"
        )

    def test_big_integer_flag_combination_4(self):
        class Secret(str):
            pass

        def default(obj):
            if isinstance(obj, Secret):
                return "******"
            raise TypeError

        assert (
            orjson.dumps(
                Secret("zxc"),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_SUBCLASS,
                default=default,
            )
            == b'"******"'
        )

        # OPT_PASSTHROUGH_DATACLASS
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_DATACLASS,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_DATACLASS,
            )
            == b"-100000000000000000001"
        )

    def test_big_integer_flag_combination_5(self):
        @dataclasses.dataclass
        class User:
            id: str
            name: str
            password: str

        def default(obj):
            if isinstance(obj, User):
                return {"id": obj.id, "name": obj.name}
            raise TypeError

        assert (
            orjson.dumps(
                User("3b1", "asd", "zxc"),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_PASSTHROUGH_DATACLASS,
                default=default,
            )
            == b'{"id":"3b1","name":"asd"}'
        )

    def test_big_integer_flag_combination_6(self):
        """
        OPT_BIG_INTEGER can be combined with other options 4
        """

        # OPT_SERIALIZE_DATACLASS
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_DATACLASS,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_DATACLASS,
            )
            == b"-100000000000000000001"
        )

        # OPT_SERIALIZE_NUMPY
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_NUMPY,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_NUMPY,
            )
            == b"-100000000000000000001"
        )

        # OPT_SERIALIZE_UUID
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_UUID,
            )
            == b"100000000000000000001"
        )
        assert (
            orjson.dumps(
                -100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_UUID,
            )
            == b"-100000000000000000001"
        )
        assert (
            orjson.dumps(
                uuid.UUID("12345678-1234-5678-1234-567812345678"),
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_UUID,
            )
            == b'"12345678-1234-5678-1234-567812345678"'
        )

        # OPT_STRICT_INTEGER
        assert (
            orjson.dumps(
                100000000000000000001,
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_STRICT_INTEGER,
            )
            == b"100000000000000000001"
        )

    @pytest.mark.skipif(numpy is None, reason="numpy is not installed")
    def test_big_integer_flag_combination_numpy(self):
        """
        OPT_BIG_INTEGER can be combined with numpy serialization options
        """
        # OPT_SERIALIZE_NUMPY with numpy.float64
        assert (
            orjson.dumps(
                numpy.float64(100000000000000000001),  # type: ignore
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_NUMPY,
            )
            == b"1e20"
        )
        assert (
            orjson.dumps(
                numpy.float64(-100000000000000000001),  # type: ignore
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_NUMPY,
            )
            == b"-1e20"
        )

        # OPT_SERIALIZE_NUMPY with numpy.ndarray
        assert (
            orjson.dumps(
                numpy.array([100000000000000000001, 123], dtype=numpy.float64),  # type: ignore
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_NUMPY,
            )
            == b"[1e20,123.0]"
        )
        assert (
            orjson.dumps(
                numpy.array([-100000000000000000001, 321], dtype=numpy.float64),  # type: ignore
                option=orjson.OPT_BIG_INTEGER | orjson.OPT_SERIALIZE_NUMPY,
            )
            == b"[-1e20,321.0]"
        )
